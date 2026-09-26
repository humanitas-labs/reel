# Tape Documentation

| Document             | Purpose                                                                                                                                                                                                              |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `strip-treemap.html` | Treemap of the repository at the fork point (Cap `40f44a803`), sized by source lines, files, or bytes. Hatched red is what Tape deleted; a hatched slice on a solid tile is the share trimmed inside a kept package. Self-contained; open in a browser. |
| `strip-treemap.png`  | Static render of the treemap. Regenerate with the command below.                                                                                                                                                     |
| `tape.png`           | The cassette artwork used at the top of the README.                                                                                                                                                                  |

Agent instructions are in `AGENTS.md`. The history of the fork is in the git log.

Regenerate the PNG:

```
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --hide-scrollbars --force-device-scale-factor=2 --window-size=1600,900 --virtual-time-budget=6000 --screenshot=docs/strip-treemap.png "file://$PWD/docs/strip-treemap.html?export"
```
