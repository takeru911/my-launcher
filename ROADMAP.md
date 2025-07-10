# My Launcher Roadmap

## Current Status (v0.1.0)

### ✅ Completed Features
- Basic launcher UI with egui/eframe
- Window enumeration and switching
- Window thumbnail previews
- Search modes (General/Windows)
- URL opening
- Command execution
- Google search fallback
- Keyboard navigation
- Cross-compilation support (Linux → Windows)
- Test infrastructure with mocks

### 🐛 Known Issues
- One failing test: `test_window_search_with_prefix` (result ordering)
- `scale_image` function unused (warning)
- No persistent window position/size
- No configuration system

## Short-term Goals (v0.2.0)

### 🎯 High Priority
1. **Global Hotkey**
   - [ ] Implement Win+Space or similar to show/hide launcher
   - [ ] Use Windows hotkey API
   - [ ] Background process management

2. **Application Search**
   - [ ] Index Start Menu entries
   - [ ] Search installed applications
   - [ ] Cache application metadata
   - [ ] Application icons

3. **Search Improvements**
   - [ ] Fuzzy matching algorithm
   - [ ] Search result ranking
   - [ ] Search history
   - [ ] Recent/frequent items

4. **Performance**
   - [ ] Optimize window enumeration
   - [ ] Async thumbnail generation
   - [ ] Smarter caching strategies

### 🔧 Technical Debt
- [ ] Fix failing test case
- [ ] Remove unused code warnings
- [ ] Improve error handling and logging
- [ ] Add proper application icon

## Medium-term Goals (v0.3.0)

### 💡 Features
1. **File Search**
   - [ ] Quick file search using Everything API or similar
   - [ ] Recent files tracking
   - [ ] File preview in results

2. **Calculator Mode**
   - [ ] Basic arithmetic
   - [ ] Unit conversions
   - [ ] Currency conversion (with API)

3. **System Commands**
   - [ ] Sleep, restart, shutdown
   - [ ] Lock screen
   - [ ] Empty recycle bin

4. **Customization**
   - [ ] Configurable hotkeys
   - [ ] Theme selection
   - [ ] Result display options
   - [ ] Search provider preferences

### 🏗️ Architecture
- [ ] Plugin system design
- [ ] Configuration management (TOML/JSON)
- [ ] IPC for single instance enforcement
- [ ] Better platform abstraction

## Long-term Vision (v1.0.0)

### 🚀 Advanced Features
1. **Plugin System**
   - [ ] Plugin API definition
   - [ ] Dynamic loading
   - [ ] Plugin marketplace/repository
   - [ ] Example plugins

2. **Workflow Automation**
   - [ ] Multi-step commands
   - [ ] Macro recording
   - [ ] Conditional actions
   - [ ] Integration with AutoHotkey

3. **AI Integration**
   - [ ] Natural language commands
   - [ ] Smart suggestions
   - [ ] Context awareness

4. **Cross-platform Support**
   - [ ] macOS implementation
   - [ ] Linux implementation
   - [ ] Consistent behavior across platforms

### 📊 Analytics & Learning
- [ ] Usage analytics (privacy-respecting)
- [ ] Machine learning for result ranking
- [ ] Personalized shortcuts
- [ ] Predictive search

## Implementation Priority

### Phase 1: Core Improvements (1-2 weeks)
1. Fix test failures
2. Add global hotkey
3. Implement application search
4. Improve search algorithm

### Phase 2: Essential Features (3-4 weeks)
1. File search
2. Calculator mode
3. Configuration system
4. Search history

### Phase 3: Polish & Performance (2-3 weeks)
1. Performance optimization
2. Better error handling
3. Installer/distribution
4. Documentation

### Phase 4: Extensibility (4-6 weeks)
1. Plugin system
2. API design
3. Example plugins
4. Developer documentation

## Technical Considerations

### Performance Targets
- Startup time: < 100ms
- Search response: < 50ms
- Memory usage: < 50MB idle
- Window enumeration: < 10ms

### Code Quality Goals
- Test coverage: > 80%
- Zero clippy warnings
- Documented public API
- Example code for extensions

### Distribution
- [ ] Windows installer (MSI/NSIS)
- [ ] Chocolatey package
- [ ] Scoop manifest
- [ ] Auto-update mechanism

## Community & Contribution

### Documentation Needs
- [ ] User guide
- [ ] Developer guide
- [ ] Plugin development guide
- [ ] Contribution guidelines

### Community Building
- [ ] GitHub issues template
- [ ] Discord/Matrix channel
- [ ] Blog posts about architecture
- [ ] Video tutorials

## Success Metrics

1. **Performance**: Faster than existing alternatives
2. **Usability**: Intuitive without documentation
3. **Extensibility**: Easy to add new features
4. **Reliability**: No crashes in normal use
5. **Community**: Active contributors and users

## Notes for Next Session

When continuing development:
1. Start by running tests to ensure environment is set up
2. Check `git status` for any uncommitted changes
3. Review this roadmap for next priority item
4. Update CLAUDE.md if any new patterns emerge
5. Keep architecture documentation in sync with changes