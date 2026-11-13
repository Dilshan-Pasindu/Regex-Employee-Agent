using System.ComponentModel;
using System.Text.Json;
using EmployeeMcpServer.DTOs;
using EmployeeMcpServer.Services;
using ModelContextProtocol.Server;

namespace EmployeeMcpServer.Tools;

[McpServerToolType]
public sealed class EmployeeTools
{
    private readonly EmployeeService _service;

    private static readonly JsonSerializerOptions JsonOpts = new()
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    private static string Serialize(object? data) => JsonSerializer.Serialize(data, JsonOpts);

    public EmployeeTools(EmployeeService service)
    {
        _service = service;
    }

    // -------------------------------------------------------------------------
    // Tool 1: get_all_employees
    // -------------------------------------------------------------------------
    [McpServerTool(Name = "get_all_employees")]
    [Description("Return all employee records including their department and designation names.")]
    public async Task<string> GetAllEmployees()
    {
        var result = await _service.GetAllEmployeesAsync();
        return Serialize(result);
    }

    // -------------------------------------------------------------------------
    // Tool 2: get_employee_by_id
    // -------------------------------------------------------------------------
    [McpServerTool(Name = "get_employee_by_id")]
    [Description("Retrieve one employee by their EmployeeID.")]
    public async Task<string> GetEmployeeById(
        [Description("The integer EmployeeID to look up (e.g. 1001).")] int employeeId)
    {
        var result = await _service.GetEmployeeByIdAsync(employeeId);
        return Serialize(result);
    }

    // -------------------------------------------------------------------------
    // Tool 3: search_employees
    // -------------------------------------------------------------------------
    [McpServerTool(Name = "search_employees")]
    [Description("Search employees by optional filters: firstName, lastName, departmentName, designationName, email, or status.")]
    public async Task<string> SearchEmployees(
        [Description("Filter by first name (partial, case-insensitive).")] string? firstName = null,
        [Description("Filter by last name (partial, case-insensitive).")] string? lastName = null,
        [Description("Filter by department name (partial, case-insensitive).")] string? departmentName = null,
        [Description("Filter by designation name (partial, case-insensitive).")] string? designationName = null,
        [Description("Filter by email address (partial, case-insensitive).")] string? email = null,
        [Description("Filter by status: Active, Inactive, or On Leave.")] string? status = null)
    {
        var input = new SearchEmployeesInput
        {
            FirstName       = firstName,
            LastName        = lastName,
            DepartmentName  = departmentName,
            DesignationName = designationName,
            Email           = email,
            Status          = status
        };
        var result = await _service.SearchEmployeesAsync(input);
        return Serialize(result);
    }

    // -------------------------------------------------------------------------
    // Tool 4: get_department_list
    // -------------------------------------------------------------------------
    [McpServerTool(Name = "get_department_list")]
    [Description("Return all departments with their ID, name, description, and employee count.")]
    public async Task<string> GetDepartmentList()
    {
        var result = await _service.GetDepartmentListAsync();
        return Serialize(result);
    }

    // -------------------------------------------------------------------------
    // Tool 5: get_employee_count
    // -------------------------------------------------------------------------
    [McpServerTool(Name = "get_employee_count")]
    [Description("Return the total employee count, optionally filtered by department name or status.")]
    public async Task<string> GetEmployeeCount(
        [Description("Optional: filter by department name (exact match, case-insensitive).")] string? department = null,
        [Description("Optional: filter by status (Active, Inactive, On Leave).")] string? status = null)
    {
        var result = await _service.GetEmployeeCountAsync(department, status);
        return Serialize(result);
    }

    // -------------------------------------------------------------------------
    // Tool 6: add_employee
    // -------------------------------------------------------------------------
    [McpServerTool(Name = "add_employee")]
    [Description("Insert a new employee record after validating all required fields.")]
    public async Task<string> AddEmployee(
        [Description("Employee first name (required).")] string firstName,
        [Description("Employee last name (required).")] string lastName,
        [Description("Employee email address — must be unique (required).")] string email,
        [Description("Department ID (integer, required). Use get_department_list to find valid IDs.")] int departmentId,
        [Description("Designation ID (integer, required). Use get_designation_list to find valid IDs.")] int designationId,
        [Description("Salary as a decimal number >= 0 (required).")] decimal salary,
        [Description("Hire date in yyyy-MM-dd format (required).")] string hireDate,
        [Description("Optional phone number.")] string? phone = null,
        [Description("Employment status: Active (default), Inactive, or On Leave.")] string status = "Active")
    {
        var input = new AddEmployeeInput
        {
            FirstName     = firstName,
            LastName      = lastName,
            Email         = email,
            Phone         = phone,
            DepartmentId  = departmentId,
            DesignationId = designationId,
            Salary        = salary,
            HireDate      = hireDate,
            Status        = status
        };
        var result = await _service.AddEmployeeAsync(input);
        return Serialize(result);
    }

    // -------------------------------------------------------------------------
    // Tool 7: update_employee
    // -------------------------------------------------------------------------
    [McpServerTool(Name = "update_employee")]
    [Description("Update an existing employee's fields. Only supply the fields you want to change.")]
    public async Task<string> UpdateEmployee(
        [Description("The EmployeeID of the employee to update (required).")] int employeeId,
        [Description("New first name (optional).")] string? firstName = null,
        [Description("New last name (optional).")] string? lastName = null,
        [Description("New email address — must be unique (optional).")] string? email = null,
        [Description("New phone number (optional).")] string? phone = null,
        [Description("New Department ID (optional).")] int? departmentId = null,
        [Description("New Designation ID (optional).")] int? designationId = null,
        [Description("New salary >= 0 (optional).")] decimal? salary = null,
        [Description("New hire date in yyyy-MM-dd format (optional).")] string? hireDate = null,
        [Description("New status: Active, Inactive, or On Leave (optional).")] string? status = null)
    {
        var input = new UpdateEmployeeInput
        {
            EmployeeId    = employeeId,
            FirstName     = firstName,
            LastName      = lastName,
            Email         = email,
            Phone         = phone,
            DepartmentId  = departmentId,
            DesignationId = designationId,
            Salary        = salary,
            HireDate      = hireDate,
            Status        = status
        };
        var result = await _service.UpdateEmployeeAsync(input);
        return Serialize(result);
    }

    // -------------------------------------------------------------------------
    // Helper Tool: get_designation_list
    // Required by Rust to resolve designation name → ID before add_employee
    // -------------------------------------------------------------------------
    [McpServerTool(Name = "get_designation_list")]
    [Description("Return all designations with their ID, name, and description. Use to resolve designation names to IDs.")]
    public async Task<string> GetDesignationList()
    {
        var result = await _service.GetDesignationListAsync();
        return Serialize(result);
    }
}
