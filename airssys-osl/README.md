# airssys-osl (OS Abstraction Layer)

This component will handle all low-level OS system programming, enhanced with activity logs and robust security policies

This component will handle these important and common system activities:
- Filesystem management
- Process management
- Network management
- Utils management
    - Something like calling other programs such as: `docker`, or `gh (github cli)`

## Motivation

The reason why I think I need this component is inspired by `airs-mcpserver-fs` project. This MCP tool provides access for an AI model so they can access the OS local environment to manipulate some filesystem (i/o), such as reading or writing a file. This MCP tool actually already provides a good enough security validator that tries to prevent any harmful activities, including avoiding any access to binary files, through its custom configurations.

I'm thinking of continuing to create other MCP tools that may need direct access to OS environments, such as running or stopping Docker, but I think I need more robust security, and also for its low-level system programming management. Actually, I can just use direct OS `Command` or `std::fs` from `Rust`, but I'm thinking to provides more controllable environments, such as:

- Monitoring commands, processes or activities
- More robust security policies, like `ACL` or `RBAC`

Based on these needs, I'm thinking of providing a high-level `OS Abstraction Layer (OSL)` that try to abstracting all of possible solutions and also provides `OS Middleware Layer`:

- Activity logs
- Robust security framework

## Architecture

![airssys-osl-arch](./assets/image.png)

### Building Blocks

#### Airssys OSL API

Provides high-level API methods or functions used by the caller to access OS activities, such as creating a new file or executing some OS processes

#### Airssys OSL Middleware

The `Middleware` component is a layer in the middle of the process between high-level APIs and their low-level OS executor. Our `OSL Framework` will pass through all requested activities to all available middleware , if there is an error on some middleware's processes, it will stop the request and throw the error.

Provides default three middlewares:

- `Logger`
- `Security`

Before each of the requested actions/activities is executed, it must go through this layer to log the activity and check for the security allowances. If it passed security check successfully, it will be forwarding to the *executor*

#### Airssys OSL Executor

Once a request passes security validations and has already been registered as a new `Runtime Process`, it will be *forwarded* to its specific executor based on activity or action type. On this layer, it will execute directly to the OS low-level executor through specific Rust OS executors. 

The main purpose of the executor must be modular, meaning that we can customize executors in the future, such as:

- Filesystem Executor
- Process Executor
- Network Executor
- Utils Executor

And all of those executors must implement the same `Executor` trait, so the `OSL Framework` can call them in the same way.