[![dependency status](https://deps.rs/repo/github/scott223/planepilot/status.svg)](https://deps.rs/repo/github/scott223/planepilot)
[![lines count](https://img.shields.io/endpoint?url=https://ghloc.vercel.app/api/scott223/planepilot/badge?filter=.rs$,.toml$)](https://ghloc.vercel.app/scott223/planepilot?filter=.rs$,.toml$)

# planepilot
xplane11 logger and (future) autopilot

```mermaid
---
title: PlanePilot
---
classDiagram
    DataServer <|-- PlaneConnector : sends plane state
    DataDashboard <|-- DataServer : dashboard retrieves all data
    PlanePilot <|-- PlanePilotUI : sends user inputs
    PlanePilot --|> PlanePilotUI : retrieves plane state
    PlaneConnector <|-- PlanePilot : sends setpoints
    PlaneConnector --|> PlanePilot : retrieves plane state
    DataServer <|-- PlanePilot : sends setpoints
    X-Plane11 <|-- PlaneConnector : sends setpoints
    X-Plane11 --|> PlaneConnector : retrieves plane state
    class DataServer{
        +Channels
        +Data
    }
    class DataDashboard {

    }
    class PlaneConnector{

    }
    class PlanePilotUI {

    }
    class X-Plane11 {

    }
```

## Tech stack

### PlaneConnector
* Rust
  * Tokio async
  * UDP sockets   

### DataDashboard
* React
  * Redux Saga
  * Rechart

###  Dataserver
* Rust
  * Tokio async
  * Axum
  * Sqlx (sqlite)
