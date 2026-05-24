pub(crate) fn simple_diff(current: &str, next: &str) -> String {
    if current == next {
        return "No changes.".to_owned();
    }
    let current_lines = current.lines().collect::<Vec<_>>();
    let next_lines = next.lines().collect::<Vec<_>>();
    let ops = diff_ops(&current_lines, &next_lines);
    let mut out = String::from("--- current\n+++ preview\n");
    let context = 3;
    let mut index = 0;
    while index < ops.len() {
        while index < ops.len() && matches!(ops[index], DiffOp::Equal(_)) {
            index += 1;
        }
        if index >= ops.len() {
            break;
        }

        let hunk_start = index.saturating_sub(context);
        let mut hunk_end = index;
        let mut trailing_equal = 0;
        while hunk_end < ops.len() {
            if matches!(ops[hunk_end], DiffOp::Equal(_)) {
                trailing_equal += 1;
                if trailing_equal > context {
                    break;
                }
            } else {
                trailing_equal = 0;
            }
            hunk_end += 1;
        }
        let hunk_end = hunk_end.min(ops.len());
        let (old_start, old_count, new_start, new_count) = hunk_range(&ops, hunk_start, hunk_end);
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            old_start, old_count, new_start, new_count
        ));
        for op in &ops[hunk_start..hunk_end] {
            match op {
                DiffOp::Equal(line) => {
                    out.push(' ');
                    out.push_str(line);
                    out.push('\n');
                }
                DiffOp::Delete(line) => {
                    out.push('-');
                    out.push_str(line);
                    out.push('\n');
                }
                DiffOp::Insert(line) => {
                    out.push('+');
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
        index = hunk_end;
    }
    out
}
#[derive(Debug, Clone, Copy)]
enum DiffOp<'a> {
    Equal(&'a str),
    Delete(&'a str),
    Insert(&'a str),
}

fn diff_ops<'a>(current: &[&'a str], next: &[&'a str]) -> Vec<DiffOp<'a>> {
    let mut prefix = 0;
    while prefix < current.len() && prefix < next.len() && current[prefix] == next[prefix] {
        prefix += 1;
    }

    let mut suffix = 0;
    while suffix + prefix < current.len()
        && suffix + prefix < next.len()
        && current[current.len() - 1 - suffix] == next[next.len() - 1 - suffix]
    {
        suffix += 1;
    }

    let old_mid = &current[prefix..current.len() - suffix];
    let new_mid = &next[prefix..next.len() - suffix];
    let mut ops = current[..prefix]
        .iter()
        .map(|line| DiffOp::Equal(line))
        .collect::<Vec<_>>();
    ops.extend(diff_middle(old_mid, new_mid));
    ops.extend(
        current[current.len() - suffix..]
            .iter()
            .map(|line| DiffOp::Equal(line)),
    );
    ops
}

fn diff_middle<'a>(current: &[&'a str], next: &[&'a str]) -> Vec<DiffOp<'a>> {
    if current.is_empty() {
        return next.iter().map(|line| DiffOp::Insert(line)).collect();
    }
    if next.is_empty() {
        return current.iter().map(|line| DiffOp::Delete(line)).collect();
    }

    let width = next.len() + 1;
    let mut lengths = vec![0usize; (current.len() + 1) * width];
    for old_index in (0..current.len()).rev() {
        for new_index in (0..next.len()).rev() {
            let slot = old_index * width + new_index;
            lengths[slot] = if current[old_index] == next[new_index] {
                lengths[(old_index + 1) * width + new_index + 1] + 1
            } else {
                lengths[(old_index + 1) * width + new_index]
                    .max(lengths[old_index * width + new_index + 1])
            };
        }
    }

    let mut ops = Vec::new();
    let mut old_index = 0;
    let mut new_index = 0;
    while old_index < current.len() && new_index < next.len() {
        if current[old_index] == next[new_index] {
            ops.push(DiffOp::Equal(current[old_index]));
            old_index += 1;
            new_index += 1;
        } else if lengths[(old_index + 1) * width + new_index]
            >= lengths[old_index * width + new_index + 1]
        {
            ops.push(DiffOp::Delete(current[old_index]));
            old_index += 1;
        } else {
            ops.push(DiffOp::Insert(next[new_index]));
            new_index += 1;
        }
    }
    ops.extend(current[old_index..].iter().map(|line| DiffOp::Delete(line)));
    ops.extend(next[new_index..].iter().map(|line| DiffOp::Insert(line)));
    ops
}

fn hunk_range(ops: &[DiffOp<'_>], start: usize, end: usize) -> (usize, usize, usize, usize) {
    let old_start = 1 + ops[..start]
        .iter()
        .filter(|op| !matches!(op, DiffOp::Insert(_)))
        .count();
    let new_start = 1 + ops[..start]
        .iter()
        .filter(|op| !matches!(op, DiffOp::Delete(_)))
        .count();
    let mut old_count = 0;
    let mut new_count = 0;
    for op in &ops[start..end] {
        match op {
            DiffOp::Equal(_) => {
                old_count += 1;
                new_count += 1;
            }
            DiffOp::Delete(_) => old_count += 1,
            DiffOp::Insert(_) => new_count += 1,
        }
    }
    (old_start, old_count, new_start, new_count)
}
