# Contributing to BevySki

Thank you for your interest in contributing to BevySki! This project is a community effort to preserve and modernize the classic MacSki game.

## How to Contribute

### Reporting Bugs
- Use the GitHub issue tracker
- Include your OS and Rust version
- Describe expected vs actual behavior
- Provide steps to reproduce

### Suggesting Features
- Check if it existed in the original MacSki
- Explain how it would improve the game
- Consider cross-platform compatibility

### Code Contributions

1. **Fork and Clone**
   ```bash
   git fork https://github.com/YOUR_USERNAME/BevySki
   git clone https://github.com/YOUR_USERNAME/BevySki
   cd BevySki
   ```

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Changes**
   - Follow Rust style guidelines (use `cargo fmt`)
   - Add comments referencing original MacSki functions where applicable
   - Test your changes

4. **Commit**
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   git commit -m "Add feature: description"
   ```

5. **Push and PR**
   ```bash
   git push origin feature/your-feature-name
   # Then create a Pull Request on GitHub
   ```

## Development Guidelines

### Code Style
- Use `cargo fmt` for formatting
- Run `cargo clippy` and address warnings
- Add documentation comments for public APIs
- Reference original MacSki functions in comments

### Project Philosophy
1. **Faithful Recreation**: Stay true to original game mechanics
2. **Modern Standards**: Use Bevy idioms and Rust best practices  
3. **Cross-Platform**: Work on Windows, macOS, Linux
4. **Open Source**: Free and accessible to everyone

### Reverse Engineering Ethics
- This is a **clean-room** implementation
- Do NOT include original game assets
- Do NOT copy decompiled code verbatim
- DO reference function behavior and game mechanics
- DO implement equivalent functionality in Rust/Bevy

### Architecture
- **ECS Pattern**: Use Bevy's Entity-Component-System
- **Systems**: One responsibility per system
- **Resources**: For global state and configuration
- **Components**: Pure data, no logic

### Testing
- Add tests for game logic (physics, collisions)
- Manual testing for gameplay feel
- Test on multiple platforms when possible

## Priority Areas

### High Priority
- [ ] Sprite rendering (skier, obstacles)
- [ ] Complete jump mechanics
- [ ] Sound effects system
- [ ] Score tracking
- [ ] Main menu

### Medium Priority
- [ ] Course editor UI
- [ ] Multiple game modes
- [ ] Weather effects
- [ ] Save/load system

### Low Priority
- [ ] Multiplayer
- [ ] Achievements
- [ ] Leaderboards
- [ ] Web build (WASM)

## Resources

- [Original MacSki Analysis](REVERSE_ENGINEERING.md)
- [Bevy Documentation](https://bevyengine.org/)
- [Rust Book](https://doc.rust-lang.org/book/)

## Questions?

Open an issue for discussion or reach out to maintainers.

## License

By contributing, you agree your contributions will be licensed under the MIT License.
