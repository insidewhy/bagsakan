# bagsakan

This is a rust project to implement static validation functions based on typescript interfaces.

Unlike other projects that need to use a transformer via `ts-patch`, this code statically generates one file containing validators which can be viewed by the user and stored in the repo.
The file containing the validator functions must be recreated if any interfaces are changed or new interface validators are added to the code and can be stored in the git repo.

This project has a toml configuration file which looks like this (the following shows the defaults):

```toml
validatorPattern = "validate%(type)"
sourceFiles = "src/**/*.ts"
validatorFile = "src/validators.ts"
```

- Uses the `oxc-parser` parse all typescript files that match `sourceFiles`.
- Finds functions that match `validatorPattern` where `%(type)` matches `[a-z][A-Z]+` and is used to identify which `interface` in the typescript code is being validated.
- Generates a function with the same name as the validator pattern match that produces code to statically verify the pattern.
- Stores all functions it found, in alphabetical order, in the `validatorFile` path.
