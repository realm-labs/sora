# Design Notes

These notes explain the architectural choices behind Sora.

The short version is that schema files are the source of truth. Excel headers, runtime bundles, generated code, and extension points are projections of the normalized schema and validated data.
