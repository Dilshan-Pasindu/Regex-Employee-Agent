-- =============================================================================
-- INTERN AGENTIC APPLICATION
-- MySQL 8.x Database Creation and Dummy Data Seeding
-- Database: InternAgenticDB
--
-- Converted from SQL Server to MySQL (Apple Silicon / macOS compatible)
-- Storage Engine : InnoDB
-- Character Set  : utf8mb4
-- Collation      : utf8mb4_unicode_ci
--
-- Safe to re-run:
--   - Creates the database if it does not exist.
--   - Creates tables if they do not exist.
--   - Seeds records only when they do not already exist.
--   - Does NOT drop tables or delete existing records.
--
-- Expected result on a fresh database:
--   6 departments | 12 designations | 20 employees (IDs 1001-1020)
-- =============================================================================

-- ---------------------------------------------------------------------------
-- 1. Create Database
-- ---------------------------------------------------------------------------
CREATE DATABASE IF NOT EXISTS InternAgenticDB
    CHARACTER SET utf8mb4
    COLLATE utf8mb4_unicode_ci;

USE InternAgenticDB;

-- ---------------------------------------------------------------------------
-- 2. Departments Table
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS Departments (
    DepartmentID   INT           NOT NULL AUTO_INCREMENT,
    DepartmentName VARCHAR(100)  NOT NULL,
    Description    VARCHAR(255)  NULL,
    CONSTRAINT PK_Departments       PRIMARY KEY (DepartmentID),
    CONSTRAINT UQ_Departments_Name  UNIQUE (DepartmentName)
) ENGINE=InnoDB
  DEFAULT CHARSET=utf8mb4
  COLLATE=utf8mb4_unicode_ci;

-- ---------------------------------------------------------------------------
-- 3. Designations Table
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS Designations (
    DesignationID   INT           NOT NULL AUTO_INCREMENT,
    DesignationName VARCHAR(100)  NOT NULL,
    Description     VARCHAR(255)  NULL,
    CONSTRAINT PK_Designations       PRIMARY KEY (DesignationID),
    CONSTRAINT UQ_Designations_Name  UNIQUE (DesignationName)
) ENGINE=InnoDB
  DEFAULT CHARSET=utf8mb4
  COLLATE=utf8mb4_unicode_ci;

-- ---------------------------------------------------------------------------
-- 4. Employees Table
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS Employees (
    EmployeeID    INT             NOT NULL AUTO_INCREMENT,
    FirstName     VARCHAR(100)    NOT NULL,
    LastName      VARCHAR(100)    NOT NULL,
    Email         VARCHAR(254)    NOT NULL,
    Phone         VARCHAR(30)     NULL,
    DepartmentID  INT             NOT NULL,
    DesignationID INT             NOT NULL,
    Salary        DECIMAL(12, 2)  NOT NULL,
    HireDate      DATE            NOT NULL,
    Status        VARCHAR(20)     NOT NULL DEFAULT 'Active',
    CONSTRAINT PK_Employees          PRIMARY KEY (EmployeeID),
    CONSTRAINT UQ_Employees_Email    UNIQUE (Email),
    CONSTRAINT FK_Employees_Dept     FOREIGN KEY (DepartmentID)
        REFERENCES Departments (DepartmentID)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT FK_Employees_Desig    FOREIGN KEY (DesignationID)
        REFERENCES Designations (DesignationID)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT CK_Employees_Salary   CHECK (Salary >= 0),
    CONSTRAINT CK_Employees_Status   CHECK (Status IN ('Active', 'Inactive', 'On Leave'))
) ENGINE=InnoDB
  AUTO_INCREMENT=1001
  DEFAULT CHARSET=utf8mb4
  COLLATE=utf8mb4_unicode_ci;

-- ---------------------------------------------------------------------------
-- 5. Indexes for common filtering and lookup operations
-- ---------------------------------------------------------------------------
CREATE INDEX IF NOT EXISTS IX_Employees_DepartmentID
    ON Employees (DepartmentID);

CREATE INDEX IF NOT EXISTS IX_Employees_DesignationID
    ON Employees (DesignationID);

CREATE INDEX IF NOT EXISTS IX_Employees_Status
    ON Employees (Status);

-- ---------------------------------------------------------------------------
-- 6. Seed Departments  (6 rows)
-- ---------------------------------------------------------------------------
INSERT INTO Departments (DepartmentName, Description)
SELECT src.DepartmentName, src.Description
FROM (
    SELECT 'IT'          AS DepartmentName, 'Information Technology Department' AS Description
    UNION ALL SELECT 'HR',          'Human Resources Department'
    UNION ALL SELECT 'Finance',     'Finance and Accounting Department'
    UNION ALL SELECT 'Sales',       'Sales and Marketing Department'
    UNION ALL SELECT 'Engineering', 'Software and Product Engineering'
    UNION ALL SELECT 'Operations',  'Business Operations Department'
) AS src
WHERE NOT EXISTS (
    SELECT 1 FROM Departments d
    WHERE d.DepartmentName = src.DepartmentName
);

-- ---------------------------------------------------------------------------
-- 7. Seed Designations  (12 rows — includes Business Analyst for Nicole Hall)
-- ---------------------------------------------------------------------------
INSERT INTO Designations (DesignationName, Description)
SELECT src.DesignationName, src.Description
FROM (
    SELECT 'Software Engineer'        AS DesignationName, 'Software development'                    AS Description
    UNION ALL SELECT 'Senior Software Engineer', 'Senior software development'
    UNION ALL SELECT 'Team Lead',                'Technical team leadership'
    UNION ALL SELECT 'Project Manager',          'Project planning and management'
    UNION ALL SELECT 'QA Engineer',              'Quality assurance and testing'
    UNION ALL SELECT 'Business Analyst',         'Business analysis'
    UNION ALL SELECT 'HR Executive',             'Human resources operations'
    UNION ALL SELECT 'Accountant',               'Financial accounting'
    UNION ALL SELECT 'Sales Executive',          'Sales operations'
    UNION ALL SELECT 'DevOps Engineer',          'Infrastructure and deployment'
    UNION ALL SELECT 'UI/UX Designer',           'User interface and experience design'
    UNION ALL SELECT 'Intern',                   'Trainee / internship position'
) AS src
WHERE NOT EXISTS (
    SELECT 1 FROM Designations g
    WHERE g.DesignationName = src.DesignationName
);

-- ---------------------------------------------------------------------------
-- 8. Seed Employees  (20 rows — IDs auto-assigned starting from 1001)
--    Uses email as the duplicate-prevention key.
--    Joins department and designation names to resolve FK IDs safely.
-- ---------------------------------------------------------------------------
INSERT INTO Employees (FirstName, LastName, Email, Phone, DepartmentID, DesignationID, Salary, HireDate, Status)
SELECT
    src.FirstName,
    src.LastName,
    src.Email,
    src.Phone,
    d.DepartmentID,
    g.DesignationID,
    src.Salary,
    src.HireDate,
    src.Status
FROM (
    SELECT 'John'        AS FirstName, 'Smith'    AS LastName, 'john.smith@example.com'        AS Email, '0771001001' AS Phone, 'IT'          AS DeptName, 'Software Engineer'        AS DesigName, CAST(85000  AS DECIMAL(12,2)) AS Salary, CAST('2024-01-15' AS DATE) AS HireDate, 'Active'   AS Status
    UNION ALL SELECT 'David',       'Miller',   'david.miller@example.com',       '0771001002', 'IT',          'Senior Software Engineer', 145000,  '2022-05-10', 'Active'
    UNION ALL SELECT 'Sarah',       'Johnson',  'sarah.johnson@example.com',      '0771001003', 'HR',          'HR Executive',              75000,   '2023-03-20', 'Active'
    UNION ALL SELECT 'Michael',     'Brown',    'michael.brown@example.com',      '0771001004', 'Finance',     'Accountant',                90000,   '2021-08-12', 'Active'
    UNION ALL SELECT 'Emily',       'Davis',    'emily.davis@example.com',        '0771001005', 'Sales',       'Sales Executive',           80000,   '2024-02-01', 'Active'
    UNION ALL SELECT 'James',       'Wilson',   'james.wilson@example.com',       '0771001006', 'Engineering', 'Team Lead',                 180000,  '2020-06-15', 'Active'
    UNION ALL SELECT 'Robert',      'Taylor',   'robert.taylor@example.com',      '0771001007', 'IT',          'DevOps Engineer',           130000,  '2022-09-01', 'Active'
    UNION ALL SELECT 'Linda',       'Anderson', 'linda.anderson@example.com',     '0771001008', 'Engineering', 'QA Engineer',               95000,   '2023-07-10', 'Active'
    UNION ALL SELECT 'William',     'Thomas',   'william.thomas@example.com',     '0771001009', 'Operations',  'Project Manager',           160000,  '2021-01-25', 'Active'
    UNION ALL SELECT 'Jessica',     'Moore',    'jessica.moore@example.com',      '0771001010', 'IT',          'UI/UX Designer',            105000,  '2023-11-05', 'Active'
    UNION ALL SELECT 'Daniel',      'Martin',   'daniel.martin@example.com',      '0771001011', 'Finance',     'Accountant',                92000,   '2022-04-18', 'Active'
    UNION ALL SELECT 'Jennifer',    'Lee',      'jennifer.lee@example.com',       '0771001012', 'HR',          'Project Manager',           125000,  '2020-10-12', 'Active'
    UNION ALL SELECT 'Christopher', 'White',    'christopher.white@example.com',  '0771001013', 'Sales',       'Sales Executive',           78000,   '2024-04-08', 'Active'
    UNION ALL SELECT 'Amanda',      'Harris',   'amanda.harris@example.com',      '0771001014', 'Engineering', 'Software Engineer',         88000,   '2024-06-01', 'Active'
    UNION ALL SELECT 'Matthew',     'Clark',    'matthew.clark@example.com',      '0771001015', 'IT',          'Intern',                    35000,   '2025-01-06', 'Active'
    UNION ALL SELECT 'Ashley',      'Lewis',    'ashley.lewis@example.com',       '0771001016', 'Engineering', 'QA Engineer',               97000,   '2022-12-15', 'Inactive'
    UNION ALL SELECT 'Andrew',      'Walker',   'andrew.walker@example.com',      '0771001017', 'IT',          'Software Engineer',         90000,   '2023-05-22', 'On Leave'
    UNION ALL SELECT 'Nicole',      'Hall',     'nicole.hall@example.com',        '0771001018', 'Operations',  'Business Analyst',          115000,  '2021-09-13', 'Active'
    UNION ALL SELECT 'Ryan',        'Allen',    'ryan.allen@example.com',         '0771001019', 'Engineering', 'Intern',                    35000,   '2025-02-10', 'Active'
    UNION ALL SELECT 'Sophia',      'Young',    'sophia.young@example.com',       '0771001020', 'IT',          'Team Lead',                 190000,  '2019-03-11', 'Active'
) AS src
INNER JOIN Departments  d ON d.DepartmentName  = src.DeptName
INNER JOIN Designations g ON g.DesignationName = src.DesigName
WHERE NOT EXISTS (
    SELECT 1 FROM Employees e
    WHERE e.Email = src.Email
);

-- ---------------------------------------------------------------------------
-- 9. Verification Summary
-- ---------------------------------------------------------------------------
SELECT
    (SELECT COUNT(*) FROM Departments)  AS TotalDepartments,
    (SELECT COUNT(*) FROM Designations) AS TotalDesignations,
    (SELECT COUNT(*) FROM Employees)    AS TotalEmployees;

-- ---------------------------------------------------------------------------
-- 10. Full employee listing with resolved names (for visual verification)
-- ---------------------------------------------------------------------------
SELECT
    e.EmployeeID,
    e.FirstName,
    e.LastName,
    e.Email,
    e.Phone,
    d.DepartmentName,
    g.DesignationName,
    e.Salary,
    e.HireDate,
    e.Status
FROM Employees e
INNER JOIN Departments  d ON d.DepartmentID  = e.DepartmentID
INNER JOIN Designations g ON g.DesignationID = e.DesignationID
ORDER BY e.EmployeeID;

-- =============================================================================
-- InternAgenticDB MySQL setup and seeding completed.
-- Expected: TotalDepartments=6, TotalDesignations=12, TotalEmployees=20
-- =============================================================================

