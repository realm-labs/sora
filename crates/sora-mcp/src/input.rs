//! Bounded, one-use continuations for modern multi-round-trip tool input.
use std::{
    collections::BTreeMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use rmcp::{
    ErrorData,
    model::{
        CallToolRequestParams, ClientCapabilities, InputRequest, InputRequiredResult, JsonObject,
    },
};
use serde_json::{Value, to_value};
use uuid::Uuid;

const INPUT_TTL: Duration = Duration::from_secs(300);
const MAX_PENDING_INPUTS: usize = 64;
const MAX_STATE_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug)]
pub(crate) struct InputSession {
    name: String,
    arguments: Option<JsonObject>,
    capabilities: Value,
    completed: BTreeMap<String, (InputRequest, Value)>,
}

impl InputSession {
    pub(crate) fn has_response(&self, key: &str) -> bool {
        self.completed.contains_key(key)
    }
}

#[derive(Debug)]
struct PendingInput {
    created: Instant,
    session: InputSession,
    key: String,
    request: InputRequest,
}

#[derive(Debug, Default)]
pub(crate) struct InputStore(Mutex<BTreeMap<String, PendingInput>>);

pub(crate) enum InputStep<T> {
    Ready(T),
    Required(InputRequiredResult),
}

impl InputStore {
    pub(crate) fn has_pending_inputs(&self) -> bool {
        let Ok(mut pending) = self.0.lock() else {
            return true;
        };
        pending.retain(|_, input| input.created.elapsed() < INPUT_TTL);
        !pending.is_empty()
    }

    pub(crate) fn resume(
        &self,
        request: &CallToolRequestParams,
        capabilities: Option<ClientCapabilities>,
    ) -> Result<InputSession, ErrorData> {
        let capabilities = to_value(capabilities).map_err(internal_error)?;
        let Some(handle) = request.request_state.as_ref() else {
            if request.input_responses.is_some() {
                return Err(invalid_input(
                    "inputResponses requires a server-issued requestState",
                ));
            }
            return Ok(InputSession {
                name: request.name.to_string(),
                arguments: request.arguments.clone(),
                capabilities,
                completed: BTreeMap::new(),
            });
        };
        let mut pending = self.0.lock().map_err(internal_error)?;
        pending.retain(|_, input| input.created.elapsed() < INPUT_TTL);
        let input = pending.get(handle).ok_or_else(|| {
            invalid_input("unknown, expired, consumed, or differently authorized requestState")
        })?;
        if input.session.name != request.name
            || input.session.arguments != request.arguments
            || input.session.capabilities != capabilities
        {
            return Err(invalid_input(
                "input retry must preserve tool, arguments, and capabilities",
            ));
        }
        let responses = request
            .input_responses
            .as_ref()
            .ok_or_else(|| invalid_input("input retry requires the requested inputResponses"))?;
        if responses.len() != 1 || !responses.contains_key(&input.key) {
            return Err(invalid_input(
                "inputResponses must match the server-issued input id",
            ));
        }
        if to_value(responses)
            .map_err(internal_error)?
            .to_string()
            .len()
            > MAX_STATE_BYTES
        {
            return Err(invalid_input("inputResponses exceeds the input size limit"));
        }
        let input = pending.remove(handle).expect("validated pending input");
        let mut session = input.session;
        session.completed.insert(
            input.key.clone(),
            (input.request, responses[&input.key].clone()),
        );
        Ok(session)
    }

    pub(crate) fn resolve(
        &self,
        session: &InputSession,
        key: &str,
        request: InputRequest,
    ) -> Result<InputStep<Value>, ErrorData> {
        if let Some((expected, response)) = session.completed.get(key) {
            if *expected != request {
                return Err(invalid_input(
                    "requested input content changed; start a new tool call",
                ));
            }
            return Ok(InputStep::Ready(response.clone()));
        }
        let size = session.name.len()
            + key.len()
            + to_value(&request)
                .map_err(internal_error)?
                .to_string()
                .len()
            + to_value(&session.arguments)
                .map_err(internal_error)?
                .to_string()
                .len()
            + session.capabilities.to_string().len()
            + session
                .completed
                .values()
                .map(|(request, response)| {
                    to_value(request).map_or(MAX_STATE_BYTES, |value| value.to_string().len())
                        + response.to_string().len()
                })
                .sum::<usize>();
        if size > MAX_STATE_BYTES {
            return Err(invalid_input(
                "input continuation exceeds the retained-state size limit",
            ));
        }
        let mut pending = self.0.lock().map_err(internal_error)?;
        pending.retain(|_, input| input.created.elapsed() < INPUT_TTL);
        if pending.len() >= MAX_PENDING_INPUTS {
            return Err(invalid_input(
                "too many pending input requests; retry after they expire",
            ));
        }
        let handle = Uuid::new_v4().to_string();
        pending.insert(
            handle.clone(),
            PendingInput {
                created: Instant::now(),
                session: session.clone(),
                key: key.to_owned(),
                request: request.clone(),
            },
        );
        Ok(InputStep::Required(InputRequiredResult::new(
            Some(BTreeMap::from([(key.to_owned(), request)])),
            Some(handle),
        )))
    }
}

fn invalid_input(message: &'static str) -> ErrorData {
    ErrorData::invalid_params(message, None)
}

fn internal_error(error: impl ToString) -> ErrorData {
    ErrorData::internal_error(error.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(deprecated)]
    use rmcp::model::ListRootsRequest;
    use serde_json::json;

    #[allow(deprecated)]
    fn roots_request() -> InputRequest {
        InputRequest::ListRoots(ListRootsRequest::default())
    }

    fn pending(store: &InputStore) -> (CallToolRequestParams, InputRequiredResult) {
        let request =
            CallToolRequestParams::new("sora_project_list").with_arguments(Default::default());
        let session = store.resume(&request, None).unwrap();
        let InputStep::Required(result) =
            store.resolve(&session, "roots", roots_request()).unwrap()
        else {
            panic!("initial input request")
        };
        (request, result)
    }

    fn retry(
        mut request: CallToolRequestParams,
        result: InputRequiredResult,
    ) -> CallToolRequestParams {
        request.request_state = result.request_state;
        request.input_responses = Some(BTreeMap::from([("roots".to_owned(), json!({"roots":[]}))]));
        request
    }

    #[test]
    fn continuation_is_one_use_and_cannot_cross_authorization_stores() {
        let store = InputStore::default();
        let other = InputStore::default();
        let (request, result) = pending(&store);
        assert!(store.has_pending_inputs());
        let request = retry(request, result);
        assert!(other.resume(&request, None).is_err());
        let session = store.resume(&request, None).unwrap();
        assert!(matches!(
            store.resolve(&session, "roots", roots_request()).unwrap(),
            InputStep::Ready(_)
        ));
        assert!(store.resume(&request, None).is_err());
        assert!(!store.has_pending_inputs());
    }

    #[test]
    fn retries_bind_tool_arguments_capabilities_and_response_id() {
        let store = InputStore::default();
        let (request, result) = pending(&store);
        let request = retry(request, result);
        let mut changed = request.clone();
        changed.name = "sora_project_open".into();
        assert!(store.resume(&changed, None).is_err());
        changed = request.clone();
        changed.arguments = Some(json!({"root_id":"other"}).as_object().unwrap().clone());
        assert!(store.resume(&changed, None).is_err());
        assert!(
            store
                .resume(&request, Some(ClientCapabilities::default()))
                .is_err()
        );
        changed = request.clone();
        changed.input_responses = Some(BTreeMap::from([(
            "script_trust".to_owned(),
            json!({"action":"accept"}),
        )]));
        assert!(store.resume(&changed, None).is_err());
        assert!(store.resume(&request, None).is_ok());
    }

    #[test]
    fn pending_inputs_expire_and_have_a_strict_store_capacity() {
        let store = InputStore::default();
        let (request, first) = pending(&store);
        for _ in 1..MAX_PENDING_INPUTS {
            pending(&store);
        }
        let session = store.resume(&request, None).unwrap();
        assert!(store.resolve(&session, "roots", roots_request()).is_err());
        for input in store.0.lock().unwrap().values_mut() {
            input.created = Instant::now() - INPUT_TTL;
        }
        assert!(!store.has_pending_inputs());
        assert!(store.resume(&retry(request, first), None).is_err());
        pending(&store);
    }

    #[test]
    fn unsolicited_and_oversized_input_responses_are_rejected() {
        let store = InputStore::default();
        let (request, result) = pending(&store);
        let mut request = retry(request, result);
        request.input_responses = Some(BTreeMap::from([(
            "roots".to_owned(),
            json!("x".repeat(MAX_STATE_BYTES)),
        )]));
        assert!(store.resume(&request, None).is_err());
        request.request_state = None;
        assert!(store.resume(&request, None).is_err());
    }
}
