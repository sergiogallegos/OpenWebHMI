# OpenWebHMI - OpenSource HMI Project @2023

## Description

The objetive is create an opensource HMI - Human Machine Interface
  - Web Development for User Interface.
  - Use Free Tools,Librarys and Frameworks.
  - Connect with Databases.
  - Connect with PLC's , include librarys like TCP/IP
  - Use as HMI or SCADA to run in any Operation System

## Features

Real Time Graphic Interface, menus, buttons, messages, graphics, reports, OEE, Visual KPIs, part traceability , alarms, recipes, predictive maintenance.

## Software Tools

- Front End : Javascript, HTML, CSS.
- Back End: Golang.
- Networking: TCP/IP
- Databases: SQLite.
- Framework: Gin
- Javascript Library: htmx 
- ORM Library: GORM.

## Sofware Project Structure

openwebhmi/
│
├── backend/
│   ├── main.go            # Entry point for the backend server
│   ├── api/               # API routes and handlers
│   ├── models/            # Data models and database schemas / GORM
│   ├── services/          # Business logic and services
│   ├── config/            # Configuration files
|   ├── tcp/               # TCP/IP for communication 
|   ├── tests/             # Create a direcotry for unit tests
|   └── docs/              # Directory for backend documentation             
│   
├── frontend/
│   ├── public/            # Static assets (HTML, CSS, images)
│   ├── src/
│   │   ├── components/    # Reusable UI components
│   │   ├── pages/         # Page components and routing
│   │   ├── api/           # Frontend API calls / htmx
│   │   ├── utils/         # Utility functions
│   │   ├── App.js         # Main App component
│   │   └── index.js       # Entry point for the frontend
|   │── docs/              # Directory for frontend documentation  
│   ├── package.json       # Frontend dependencies and scripts
│   └── ...
│
├── database/              # Database-related files (migrations, seeds, etc.)
│
├── scripts/               # Deployment and automation scripts
│
├── README.md              # Project documentation
├── .gitignore             # Git ignore rules
└── ...




## Machine Builders and System Integrators

Thanks and I hope the community support this project to be used by the industry for free.

For all the machine builders and system integrators that are looking for a modular, flexible and opensoruce HMI/SCADA that can run
on any PC + Monitor or PC with embeeded Touch Monitor.

Lets create the Next Generation of HMI for Industry 4.0 OpenSource Project.

## Maintainers
Sergio Gallegos - Repo Owner  - sergiogallegos.net

## Sponsors
-Not Sponsor at this time

### License

OpenWebHMI is [MIT licensed](./LICENSE).
