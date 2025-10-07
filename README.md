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

---

### Objectives

The objective of **FinanceR** is to design and implement a secure, feature-rich, and user-friendly personal finance management application built in Rust. Unlike many Rust-based tools designed in class that have been restricted to command-line interfaces, FinanceR introduces a graphical user interface (GUI) with back-end integration, making it accessible to both technical and non-technical users.  

FinanceR provides a centralized platform for multi-user financial management, enabling users to create secure profiles with password authentication and access a consolidated view of their accounts, similar to existing online banking systems.  

Another key goal is to deliver a comprehensive income and expense tracker. Users can log transactions manually or automate recurring entries such as bills or salaries. Categories (default or custom) will provide more granular insight into spending and income patterns, allowing for visualizations.  

Finally, FinanceR will include long-term planning tools for savings and investment. Users can project growth with recurring contributions and expected returns, supported by visualizations of compounding effects.  

By combining the safety and performance of Rust with an intuitive GUI, FinanceR fills a gap in the ecosystem, emphasizing security, usability, and actionable insights to improve financial decision-making.  

---

### Key Features

To meet these objectives, FinanceR will implement a well-defined set of features organized into categories. Each feature is designed to be modular, ensuring that development tasks can be distributed among team members while still contributing to a cohesive final product.  



#### 1. User Authentication and Profiles
- Multi-user access: FinanceR will support multiple users, each with a unique username and password combination. Passwords and financial data will be stored securely using hashing and encryption techniques. Additionally, each user’s financial data will be isolated to maintain privacy.
- Onboarding flow: First-time users will be prompted to create a profile, while returning users will log in via a clean GUI login screen. Error handling will be implemented for incorrect credentials or duplicate usernames.




#### 2. Account Management
- Multiple account types: Users can create and manage various accounts such as checking, savings, credit cards, or custom-defined accounts.
- Dashboard view: A consolidated overview of balances across all accounts will be displayed in the GUI, mirroring the layout of a typical online banking dashboard.




#### 3. Transaction Management
- Income and expense logging: Every financial activity can be recorded as a transaction. Transactions will include metadata such as date, amount, category, and account.
- Recurring transactions: Users can define recurring expenses or income sources that will automatically update balances on a daily, weekly, or monthly basis. This ensures routine payments are tracked without manual entry.
- Transfers between accounts: Support for internal transfers, such as moving money from checking to savings, distinct from external expenses or income.
- Categorization system: Each transaction can be tagged with one or more categories. Default categories will be provided, but users will be able to define custom categories as needed.



#### 4. Budgeting and Expense Tracking
- Category-specific budgets: Users can set spending limits for expense categories and income targets for revenue categories.
- Time-based tracking: Budgets can be defined on flexible timescales (daily, weekly, monthly, yearly).
- Spending indicators: The GUI will provide visual cues (like a progress bar) when a user approaches or exceeds their budget thresholds.
- Interactive dashboards: Users will see breakdowns of where their money is going through charts and tables, helping them make data-driven spending adjustments.




#### 5. Savings and Investment Tools
- Savings calculator: Users can input recurring monthly contributions and project how much will be accumulated over time.
- Investment simulator and visualization: Users can experiment with hypothetical investment scenarios by providing expected rates of return. FinanceR will generate growth curves showing projected outcomes. These tools will be supported with line and bar charts that clearly demonstrate growth over months or years, highlighting the effects of compounding.
  



#### 6. Reporting and Visualization
- Flexible reports: Users can generate reports based on account activity, income sources, or expense categories. Reports can be filtered by time period and granularity (daily, monthly, yearly).
- Graphical summaries: The GUI will display pie charts for expense breakdowns, line charts for income trends, and comparative bar charts for budgets.




---

### Tentative Plan

#### **Project Phase Plan**

The development of FinanceR will follow a modular and time-efficient approach to ensure all major features are implemented and tested before the December 15th deadline. Each phase builds on the previous one , ensuring the system remains stable, maintainable, and cohesive throughout development. The GUI will be built as each feature is completed in the backend.

---

#### **Phase 1 – Foundation Setup (October 14 – October 20)**
The project will begin with the initial setup of the Rust environment and Git repository. The database schema for users, accounts, and transactions will be designed and tested for consistency.  
For the purposes of this project, we will use a **local database**.  
This phase will also establish the basic file structure, configuration, and data models needed for modular development. Input/output handling will be set up to maintain and show user presence and initial session state.

---

#### **Phase 2 – Authentication and User Profiles (October 21 – October 31)**
This phase focuses on implementing secure user registration and login. Features such as password hashing, input validation, and error handling will ensure data integrity and user privacy. Each user will have a unique, isolated data profile, forming the core of the multi-user architecture.

---

#### **Phase 3 – Account and Transaction Management (November 1 – November 10)**
Users will be able to create and manage multiple account types, such as checking, savings, or custom accounts. Transaction logging, internal transfers, and recurring transactions will be implemented to automate common financial actions and maintain accurate account balances.

---

#### **Phase 4 – Budgeting and Expense Tracking (November 11 – November 20)**
The budgeting module will allow users to set category-specific spending limits and income goals. The system will track these metrics over flexible timescales and alert users when thresholds are approached. Recurring income and expenses will be integrated for accurate financial forecasting.

---

#### **Phase 5 – Reporting and Visualization (November 21 – November 30)**
Reporting tools will enable users to generate text-based summaries by date, account, or category. Financial insights such as expense breakdowns and income trends will be displayed clearly to support user decision-making.

---

#### **Phase 6 – Testing, Integration, and Documentation (December 1 – December 15)**
The final phase will focus on end-to-end testing, debugging, and performance optimization. Full system integration will be verified across modules. Comprehensive documentation—including the **User Guide Write-Up**, reproducibility report, and final demo preparation—will be completed before submission.

---

#### **Team Responsibilities**

| **Team Member** | **Responsibilities** | **Additional Deliverables** |
|------------------|----------------------|------------------------------|
| **Mohammad** | Focuses on core infrastructure and authentication, including database setup, user registration/login, and GUI setup. Ensures proper error handling and data validation across modules. | - User Guide Write-up  <br> - Demo setup and recording |
| **Vishnu** | Leads account and transaction management. Implements account creation, transaction logging, internal transfers, recurring transactions, and categorization system for financial data. | - Reproducibility Guide Write-up  <br> - Objectives and Key Features section |
| **Khantil** | Implements budgeting, savings, and reporting features, including the budgeting engine, savings/investment calculators, and text-based reporting/export functionality. | - Motivation and Objectives Write-up  <br> - Lessons Learned and Conclusion Write-up |
| **All Members** | Collaborate on initial database table schemas, conduct code reviews, and test different feature functionalities. Each feature will have one lead developer and at least one supporting developer. | On a rotational basis each team member will lead a weekly sync up to give any updates and blockers. |

### Project Proposal References
[1] Best budgeting apps in Canada for 2025, https://money.ca/managing-money/budgeting/best-budget-apps-canada (accessed Oct. 5, 2025). 

[2] Setting_tracker - crates.io: Rust package registry, https://crates.io/crates/setting_tracker (accessed Oct. 5, 2025).
