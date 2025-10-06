# Personal-Finance-Tracker

## Project Proposal

### Team Members

| Name           | Student ID |
| -------------- | ---------- |
| Khantil Desai  | 1006155161 |
| Vishnu Akundi  | 1006212028 |
| Mohammad Harun | 1005235844 |

### Motivation

In the modern era where there are countless ways that businesses are trying to encourage unnecessary spending, it is more important now, than ever before for people to be equipped with the tools necessary to manage their finances. Currently there are various tools available to track personal finances but most of them are on a for-profit driven model, which require users to make one-time or recurring purchases to use the product. [1] These types of solutions are not ideal for mass adoption. To really help the most number of people improve their finances a free tool must be developed.

This is why a Rust crate would make the most sense since they are freely available and can easily be installed and used on most operating systems. The other benefit of using Rust is that Rust removes entire classes of errors such as dangling pointers, double free errors, and null pointer dereferences. These types of errors are bound to proliferate in any legacy system as they develop, however, this proposed app will be immune to it. A key limitation of existing Rust finance tracking apps is the lack of a GUI. Current implementations are limited to command line and file based I/O. [2] This is where FinanceR will fill a gap in the current ecosystem. FinanceR will implement a simple and intuitive GUI that will allow this app to be used by as many people as possible. 


### Plan
| **Team Member** | **Responsibilities** | **Additional Deliverables** |
|------------------|----------------------|------------------------------|
| **Mohammad** | Focuses on core infrastructure and authentication, including database setup, user registration/login, and GUI setup. Ensures proper error handling and data validation across modules. | - User Guide  <br> - Demo setup and recording |
| **Vishnu** | Leads account and transaction management. Implements account creation, transaction logging, internal transfers, recurring transactions, and categorization system for financial data. | - Reproducibility Guide  <br> - Objectives and Key Features section |
| **Khantil** | Implements budgeting, savings, and reporting features, including the budgeting engine, savings/investment calculators, and text-based reporting/export functionality. | - Motivation and Objectives section  <br> - Lessons Learned and Conclusion |
| **All Members** | Collaborate on initial database table schemas, conduct code reviews, and test different feature functionalities. Each feature will have one lead developer and at least one supporting developer. | On a rotational basis each team member will lead a weekly sync up to give any updates and blockers. |

### Project Proposal References
[1] Best budgeting apps in Canada for 2025, https://money.ca/managing-money/budgeting/best-budget-apps-canada (accessed Oct. 5, 2025). 

[2] Setting_tracker - crates.io: Rust package registry, https://crates.io/crates/setting_tracker (accessed Oct. 5, 2025).