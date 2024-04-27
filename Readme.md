# Markdown Replace Tool

## Overview

The Markdown Replace Tool is a cargo tool designed to search for specific patterns in markdown files within a specified directory and replace them with absolute links. This tool is particularly useful for managing and updating links in large markdown documentation projects.

## Usage

To use this tool, you need to specify the root directory and optionally, a list of directories to ignore. Here is how you can call the tool from the command line:

```
cargo run ../../ML-Notes/  --ignore-dirs .obsidian,assets,.git
```

In this case, it'll go the `ML-Notes` directory and from there it will recursively traverse the file to find all files. Then we'll read in the file and find

Here is a gif of it in actions

![Warp Gif](./Warp.gif)
