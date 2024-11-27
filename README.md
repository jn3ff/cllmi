Need to have CLAUDE_API_KEY set in your env to use this application

to build, run the build script with ```sh build.sh```

then cllmi --help for more information


this is a stupid tool, i didn't care to handle edge cases yet, so your use has to be happypath for it to work.

1. run some command
2. run cllmi, it will suggest to you a fixed version
3. hit enter to run the command, or c, enter to copy it to your clipboard


example uses:
```
❯ kubectl logs -l app="someapp" --tail=1 | jq 'selector'
jq: error: selector/0 is not defined at <top-level>, line 1:
selector
jq: 1 compile error
❯ cllmi -c "I am hoping to extract just the message field from this structured log. it's nested under fields: {message}"

Suggested command: kubectl logs -l app="someapp" --tail=1 | jq '.fields.message'
Justification: The selector in jq needs to use dot notation to access nested fields in JSON. The dot (.) at the start indicates we're starting from the root of the JSON object, and we can access nested fields using additional dots. Since the message is nested under 'fields', we use '.fields.message'. Also, the selector should be in quotes when passed to jq.

Press Enter to execute the command, 'c' to copy to clipboard, or Ctrl+C to cancel...

<logs printed here>

------

❯ git fetsh
git: 'fetsh' is not a git command. See 'git --help'.

The most similar command is
	fetch
❯ cllmi -j
Suggested command: git fetch
Justification: The command 'fetsh' was misspelled. 'fetch' is the correct git command used to download objects and refs from a remote repository. This is commonly used to update your local repository with changes from a remote repository before performing operations like merge or rebase.

Press Enter to execute the command, 'c' to copy to clipboard, or Ctrl+C to cancel...

From <repo>
 * [new branch]          <branch>
   main       -> origin/main

❯ cllmi --help
Usage: cllmi [OPTIONS]

Options:
  -m, --model <MODEL>      Model to use. This gets sent straight to the api, so if you override, make sure it's a valid model string [default: claude-3-5-sonnet-20241022]
  -c, --context <CONTEXT>  Any contextual information about the goal of your command, to be sent to the api so it can make a better decision [default: ]
  -j, --justify            use flag to compel model to give you a justification for its selected command
  -h, --help               Print help
  -V, --version            Print version
```
