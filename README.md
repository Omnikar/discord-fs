# discord-fs

This program allows you to use a Discord bot to upload and download
files from a channel in a Discord server.

## Setup

Currently, the program loads values from a `.env` file in the current
directory. In order to work, the following values must be supplied:

* `DISCORD_FS_TOKEN`: The Discord bot token to use
* `DISCORD_FS_CHANNEL_ID`: The ID of the channel to access files in

## Usage

Invoke the program with either of the `--upload` or `--download`
flags.

* `--upload`: Given a file path, upload the file to Discord
  (splitting if needed) and print the message ID.
* `--download`: Given a message ID, download and consolidate the file
  from Discord and print the written file path.
