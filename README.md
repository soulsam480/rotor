## Rotor

terminal secrets rotator written in rust.

### Getting started

- install (todo: add to homebrew)

```
curl -fsSL https://raw.githubusercontent.com/soulsam480/rotor/refs/heads/master/install | bash
```

- run `rotor_bin init`, this does two things
  - adds the `rotor` shell function to `~/.zshrc`
  - initialized a `~/.secretsrc` file with examples if not present.
- run `rotor` after initialization, this will show available secrets and
  available values to be set

### How does it work ?

- `rotor` reads the secretsrc and asks which secret can be set
- based on the selection, it'll ask to select a option (by their label)
- then it exports that selection and the chosen value is set for the session

### Potential improvements

- directly set using `rotor set GEMINI_KEY opencode`
