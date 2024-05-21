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
