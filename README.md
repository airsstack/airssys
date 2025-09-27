# AirsSys - System Programming Components for AirsStack

`AirsSys` is a collection of system programming components designed to facilitate the development of applications within the AirsStack ecosystem. It provides essential tools and libraries for managing system resources, handling low-level operations, and ensuring efficient performance.

`Airssys` is one of the `airsstack` projects, which is designed to manage the OS system programming, `Erlang Actor Model` runtime system, and pluggable system.

This project will contains three important components:

- `airssys-osl`
- `airssys-rt`
- `airssys-wasm`

## airssys-osl (OS Layer Framework)

This component will handle all low-level OS system programming, enhanced with activity logs and robust security policies.

This component will handle these important and common system activities:

- Filesystem management
- Process management
- Network management
- Utils management
    - Something like calling other programs such as: `docker`, or `gh (github cli)`

## airssys-rt (Runtime)

This component will provide a model of the `Erlang-Actor` system. This component will provide a supervisor for its process management, which follows the `BEAM` approaches.

> [!IMPORTANT]
>
> The `airssys-rt` component is not intended to replace the `BEAM` runtime system. This component will just take the `Erlang-Actor` model as a reference to build its own actor model system and its process management. This component will provide a lightweight actor model system, which can be used to manage high-concurrent applications.

Principles:

- **Encapsulation:** An actor maintains its own private, internal state that cannot be directly accessed or modified by any other actor.
- **Asynchronous Message Passing:** Actors communicate exclusively by sending and receiving immutable messages asynchronously. There is no shared memory, which eliminates the need for complex and error-prone synchronization mechanisms like locks, thereby preventing race conditions by design.
- **Mailbox and Sequential Processing:** Each actor has a "mailbox" that queues incoming messages. The actor processes these messages one at a time, in a sequential manner, ensuring that its internal state is always consistent.

## airssys-wasm (WASM Pluggable System)

This component will provide a WebAssembly (WASM) runtime environment, enabling the execution of WASM modules within the AirsStack ecosystem. It will include features such as:

- A lightweight WASM VM for executing WASM binaries
- Integration with the AirsSys components for seamless communication
- Support for both synchronous and asynchronous execution models
- Security features to sandbox WASM modules and restrict their access to system resources

Previously, I have had thoughts related to the "isolated processes", which relate to the `cgroups` and `namespace`; unfortunately, these features are only available in Linux OS environments. These limitations made me think about avoiding these kinds of approaches.

Related with the `WASM` approach, I have found that `WASM` can be a good alternative to the "isolated processes" approach. `WASM` can provide a secure and efficient way to run code in a sandboxed environment, which can be beneficial for certain use cases.

Taken from my researches:

> [!NOTE]
>
> Today, the industry stands at the cusp of another architectural evolution, moving towards a model that promises even finer-grained modularity, unparalleled portability, and security by default. This new paradigm is centered on the WebAssembly (WASM) Component Model, a technology that reimagines software composition. It envisions applications built not from large, loosely coupled services, but from small, secure, and language-agnostic components that can be seamlessly composed like Lego bricks.
>
> This report posits that the convergence of WebAssembly, the WebAssembly System Interface (WASI), and the Component Model represents a fundamental architectural shift, enabling a new generation of polyglot, composable, and secure applications that can truly run anywhere—from the largest cloud server to the smallest edge device.

By providing WASM management, especially to build pluggable systems based on a host-guest system. Besides, by following WASM, it already runs inside a sandboxed environment with a deny-by-default security policy, meaning that using WASM it already solve my previous thoughts about "isolated processes".