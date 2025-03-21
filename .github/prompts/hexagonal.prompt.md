# Hexagonal Architecture Implementation Guide

## Core Principles
- Business Logic is isolated at center, separated from UI/infrastructure
- Dependencies flow from outside (adapters) to inside (domain), never reverse
- Communication occurs through Ports (interfaces) and Adapters (implementations)

## Structure
- **Domain Layer** (center): Contains business logic, entities, use cases
- **Ports**: Interfaces defined by domain that specify how outside world interacts
  - Primary/Driving Ports: For outside-to-inside communication (user actions)
  - Secondary/Driven Ports: For inside-to-outside communication (infrastructure)
- **Adapters**: Implementations connecting to ports
  - Primary/Driving Adapters: User-side (UI, API controllers, CLI)
  - Secondary/Driven Adapters: Server-side (repositories, external services)

## Implementation Tips
- Define domain model first without technical dependencies
- Use dependency injection to connect adapters to ports
- Keep domain logic pure and focused on business rules
- Package structure: separate domain, ports, and adapters
- Enforce architectural boundaries with access modifiers