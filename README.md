# OpenWebHMI - OpenSource HMI Project @2023

Table of Contents
=======================

* [Description](#description)
* [Features](#features)
* [Software Tools](#software-tools)
* [Project Structure](#project-structure)
* [Machine Builders & System Integrators](#machine-builders-and-system-integrators)
* [Maintainers](#maintainers)
* [Licensing](#licensing)

Description
------

The objetive is create an opensource HMI - Human Machine Interface
  - Web Development for User Interface.
  - Use OpenSource Tools, Libraries and Frameworks.
  - Connect with Databases.
  - Connect with PLC's , include librarys like TCP/IP
    - AllenBradley, Siemens, Beckhoff, etc
  - Run in any OS (Windows, Linux, MacOS)

Features
------

Real Time Graphic Interface, Menus, Buttons, Messages, Alarms, Graphics, Reports, OEE, Visual KPIs, Part Traceability, Recipes, Predictive Maintenance and More.

Software Tools
------

- Front End : Javascript, HTML, CSS.
- Back End: Golang.
- Networking: TCP/IP
- Databases: SQLite.
- Framework: Gin
- Javascript Library: htmx 
- ORM Library: GORM.

Project Structure
------

- `backend/`: Contains the Go backend code.
  - `main.go`: Entry point for the backend server.
  - `api/`: API routes and handlers.
  - `models/`: Data models and GORM schemas.
  - `services/`: Business logic and services.
  - `config/`: Configuration files.
  - `tcp/`: TPC/IP for communication.
  - `tests/`: Directory for unit tests.
  - `docs/`: Directory for backend documentation.
  - ...

- `frontend/`: Contains the JavaScript frontend code.
  - `public/`: Static assets (HTML, CSS, images).
  - `src/`: Main source code directory.
    - `components/`: Reusable UI components.
    - `pages/`: Page components and routing.
    - `api/`: Frontend API calls (htmx).
    - `utils/`: Utility functions.
    - `App.js`: Main App component.
    - `index.js`: Entry point for the frontend.
  - `docs/`: Directory for frontend documentation.
  - `package.json`: Frontend dependencies and scripts.
  - ...

- `database/`: Database-related files (migrations, seeds, etc.).
  - `migrations/`: Database migration scripts.
  - ...

- `scripts/`: Deployment and automation scripts.
  - `deploy.sh`: Deployment script.
  - ...

- `README.md`: Project documentation.
- `.gitignore`: Git ignore rules.


Machine Builders and System Integrators
------

For all the machine builders and system integrators that are looking for a modular, flexible and OpenSource HMI with Web Technology and Update Software Tools that can run on any OS(Linux, Windows or MacOS), This is the correct place.

Let's create the Next Generation of HMI for Industry 4.0 OpenSource Project.

Contribute code
------

If you would like to become an active contributor to this project please send your commits and ideas for this project.

Maintainers
------

Sergio Gallegos - Repo Owner  - sergiogallegos.net

Sponsors
------

We don't have sponsors at this time. If you are a company that want
to sponsor this project, you are more than welcome.

Licensing
------

OpenWebHMI is released under the [MIT licensed](./LICENSE).
