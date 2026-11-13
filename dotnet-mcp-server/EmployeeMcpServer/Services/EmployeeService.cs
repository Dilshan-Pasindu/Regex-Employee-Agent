using EmployeeMcpServer.Data;
using EmployeeMcpServer.DTOs;
using EmployeeMcpServer.Models;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;

namespace EmployeeMcpServer.Services;

public class EmployeeService
{
    private static readonly string[] AllowedStatuses = ["Active", "Inactive", "On Leave"];

    private readonly EmployeeDbContext _db;
    private readonly ILogger<EmployeeService> _logger;

    public EmployeeService(EmployeeDbContext db, ILogger<EmployeeService> logger)
    {
        _db = db;
        _logger = logger;
    }

    // -------------------------------------------------------------------------
    // Helper: map Employee entity → EmployeeDto
    // -------------------------------------------------------------------------
    private static EmployeeDto ToDto(Employee e) => new()
    {
        EmployeeId     = e.EmployeeId,
        FirstName      = e.FirstName,
        LastName       = e.LastName,
        Email          = e.Email,
        Phone          = e.Phone,
        DepartmentId   = e.DepartmentId,
        DepartmentName = e.Department?.DepartmentName ?? string.Empty,
        DesignationId  = e.DesignationId,
        DesignationName= e.Designation?.DesignationName ?? string.Empty,
        Salary         = e.Salary,
        HireDate       = e.HireDate.ToString("yyyy-MM-dd"),
        Status         = e.Status
    };

    // -------------------------------------------------------------------------
    // Tool 1: get_all_employees
    // -------------------------------------------------------------------------
    public async Task<ToolResult> GetAllEmployeesAsync()
    {
        try
        {
            var employees = await _db.Employees
                .Include(e => e.Department)
                .Include(e => e.Designation)
                .OrderBy(e => e.EmployeeId)
                .Select(e => ToDto(e))
                .ToListAsync();

            return ToolResult.Ok(employees, $"{employees.Count} employee(s) found.");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in GetAllEmployeesAsync");
            return ToolResult.Fail($"Database error: {ex.Message}");
        }
    }

    // -------------------------------------------------------------------------
    // Tool 2: get_employee_by_id
    // -------------------------------------------------------------------------
    public async Task<ToolResult> GetEmployeeByIdAsync(int employeeId)
    {
        try
        {
            var employee = await _db.Employees
                .Include(e => e.Department)
                .Include(e => e.Designation)
                .FirstOrDefaultAsync(e => e.EmployeeId == employeeId);

            if (employee is null)
                return ToolResult.Fail($"Employee with ID {employeeId} was not found.");

            return ToolResult.Ok(ToDto(employee));
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in GetEmployeeByIdAsync");
            return ToolResult.Fail($"Database error: {ex.Message}");
        }
    }

    // -------------------------------------------------------------------------
    // Tool 3: search_employees
    // -------------------------------------------------------------------------
    public async Task<ToolResult> SearchEmployeesAsync(SearchEmployeesInput input)
    {
        try
        {
            var query = _db.Employees
                .Include(e => e.Department)
                .Include(e => e.Designation)
                .AsQueryable();

            if (!string.IsNullOrWhiteSpace(input.FirstName))
                query = query.Where(e => e.FirstName.ToLower().Contains(input.FirstName.ToLower()));

            if (!string.IsNullOrWhiteSpace(input.LastName))
                query = query.Where(e => e.LastName.ToLower().Contains(input.LastName.ToLower()));

            if (!string.IsNullOrWhiteSpace(input.Email))
                query = query.Where(e => e.Email.ToLower().Contains(input.Email.ToLower()));

            if (!string.IsNullOrWhiteSpace(input.Status))
                query = query.Where(e => e.Status.ToLower() == input.Status.ToLower());

            if (!string.IsNullOrWhiteSpace(input.DepartmentName))
                query = query.Where(e => e.Department != null &&
                    e.Department.DepartmentName.ToLower().Contains(input.DepartmentName.ToLower()));

            if (!string.IsNullOrWhiteSpace(input.DesignationName))
                query = query.Where(e => e.Designation != null &&
                    e.Designation.DesignationName.ToLower().Contains(input.DesignationName.ToLower()));

            var results = await query
                .OrderBy(e => e.EmployeeId)
                .Select(e => ToDto(e))
                .ToListAsync();

            return ToolResult.Ok(results, $"{results.Count} employee(s) matched.");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in SearchEmployeesAsync");
            return ToolResult.Fail($"Database error: {ex.Message}");
        }
    }

    // -------------------------------------------------------------------------
    // Tool 4: get_department_list
    // -------------------------------------------------------------------------
    public async Task<ToolResult> GetDepartmentListAsync()
    {
        try
        {
            var departments = await _db.Departments
                .OrderBy(d => d.DepartmentId)
                .Select(d => new DepartmentDto
                {
                    DepartmentId   = d.DepartmentId,
                    DepartmentName = d.DepartmentName,
                    Description    = d.Description,
                    EmployeeCount  = d.Employees.Count
                })
                .ToListAsync();

            return ToolResult.Ok(departments, $"{departments.Count} department(s) found.");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in GetDepartmentListAsync");
            return ToolResult.Fail($"Database error: {ex.Message}");
        }
    }

    // -------------------------------------------------------------------------
    // Tool 5: get_employee_count
    // -------------------------------------------------------------------------
    public async Task<ToolResult> GetEmployeeCountAsync(string? department = null, string? status = null)
    {
        try
        {
            var query = _db.Employees
                .Include(e => e.Department)
                .AsQueryable();

            if (!string.IsNullOrWhiteSpace(department))
                query = query.Where(e => e.Department != null &&
                    e.Department.DepartmentName.ToLower() == department.ToLower());

            if (!string.IsNullOrWhiteSpace(status))
                query = query.Where(e => e.Status.ToLower() == status.ToLower());

            var count = await query.CountAsync();

            var filterDesc = new List<string>();
            if (!string.IsNullOrWhiteSpace(department)) filterDesc.Add($"department='{department}'");
            if (!string.IsNullOrWhiteSpace(status)) filterDesc.Add($"status='{status}'");
            var filterStr = filterDesc.Count > 0 ? $" (filtered by {string.Join(", ", filterDesc)})" : "";

            return ToolResult.Ok(new { Count = count, Filters = filterDesc },
                $"Total employees{filterStr}: {count}");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in GetEmployeeCountAsync");
            return ToolResult.Fail($"Database error: {ex.Message}");
        }
    }

    // -------------------------------------------------------------------------
    // Tool 6: add_employee
    // -------------------------------------------------------------------------
    public async Task<ToolResult> AddEmployeeAsync(AddEmployeeInput input)
    {
        try
        {
            // Validate required fields
            if (string.IsNullOrWhiteSpace(input.FirstName))
                return ToolResult.Fail("FirstName is required.");
            if (string.IsNullOrWhiteSpace(input.LastName))
                return ToolResult.Fail("LastName is required.");
            if (string.IsNullOrWhiteSpace(input.Email))
                return ToolResult.Fail("Email is required.");
            if (input.Salary < 0)
                return ToolResult.Fail("Salary must be zero or greater.");

            var status = string.IsNullOrWhiteSpace(input.Status) ? "Active" : input.Status;
            if (!AllowedStatuses.Contains(status))
                return ToolResult.Fail($"Invalid status '{status}'. Allowed values: Active, Inactive, On Leave.");

            if (!DateOnly.TryParse(input.HireDate, out var hireDate))
                return ToolResult.Fail($"Invalid HireDate format '{input.HireDate}'. Use yyyy-MM-dd.");

            // Validate department FK
            var deptExists = await _db.Departments.AnyAsync(d => d.DepartmentId == input.DepartmentId);
            if (!deptExists)
                return ToolResult.Fail($"DepartmentID {input.DepartmentId} does not exist.");

            // Validate designation FK
            var desigExists = await _db.Designations.AnyAsync(g => g.DesignationId == input.DesignationId);
            if (!desigExists)
                return ToolResult.Fail($"DesignationID {input.DesignationId} does not exist.");

            // Check duplicate email
            var emailExists = await _db.Employees.AnyAsync(e => e.Email.ToLower() == input.Email.ToLower());
            if (emailExists)
                return ToolResult.Fail($"An employee with email '{input.Email}' already exists.");

            var employee = new Employee
            {
                FirstName     = input.FirstName.Trim(),
                LastName      = input.LastName.Trim(),
                Email         = input.Email.Trim().ToLower(),
                Phone         = input.Phone?.Trim(),
                DepartmentId  = input.DepartmentId,
                DesignationId = input.DesignationId,
                Salary        = input.Salary,
                HireDate      = hireDate,
                Status        = status
            };

            _db.Employees.Add(employee);
            await _db.SaveChangesAsync();

            // Reload with navigation properties for the response
            await _db.Entry(employee).Reference(e => e.Department).LoadAsync();
            await _db.Entry(employee).Reference(e => e.Designation).LoadAsync();

            return ToolResult.Ok(ToDto(employee), $"Employee '{employee.FirstName} {employee.LastName}' created with ID {employee.EmployeeId}.");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in AddEmployeeAsync");
            return ToolResult.Fail($"Database error: {ex.Message}");
        }
    }

    // -------------------------------------------------------------------------
    // Tool 7: update_employee
    // -------------------------------------------------------------------------
    public async Task<ToolResult> UpdateEmployeeAsync(UpdateEmployeeInput input)
    {
        try
        {
            if (input.EmployeeId <= 0)
                return ToolResult.Fail("A valid EmployeeID is required.");

            var employee = await _db.Employees
                .Include(e => e.Department)
                .Include(e => e.Designation)
                .FirstOrDefaultAsync(e => e.EmployeeId == input.EmployeeId);

            if (employee is null)
                return ToolResult.Fail($"Employee with ID {input.EmployeeId} was not found.");

            // Apply updates only for fields that were provided
            if (!string.IsNullOrWhiteSpace(input.FirstName))
                employee.FirstName = input.FirstName.Trim();

            if (!string.IsNullOrWhiteSpace(input.LastName))
                employee.LastName = input.LastName.Trim();

            if (!string.IsNullOrWhiteSpace(input.Email))
            {
                var emailConflict = await _db.Employees.AnyAsync(e =>
                    e.Email.ToLower() == input.Email.ToLower() && e.EmployeeId != input.EmployeeId);
                if (emailConflict)
                    return ToolResult.Fail($"Email '{input.Email}' is already in use by another employee.");
                employee.Email = input.Email.Trim().ToLower();
            }

            if (input.Phone is not null)
                employee.Phone = string.IsNullOrWhiteSpace(input.Phone) ? null : input.Phone.Trim();

            if (input.DepartmentId.HasValue)
            {
                var deptExists = await _db.Departments.AnyAsync(d => d.DepartmentId == input.DepartmentId.Value);
                if (!deptExists)
                    return ToolResult.Fail($"DepartmentID {input.DepartmentId} does not exist.");
                employee.DepartmentId = input.DepartmentId.Value;
            }

            if (input.DesignationId.HasValue)
            {
                var desigExists = await _db.Designations.AnyAsync(g => g.DesignationId == input.DesignationId.Value);
                if (!desigExists)
                    return ToolResult.Fail($"DesignationID {input.DesignationId} does not exist.");
                employee.DesignationId = input.DesignationId.Value;
            }

            if (input.Salary.HasValue)
            {
                if (input.Salary.Value < 0)
                    return ToolResult.Fail("Salary must be zero or greater.");
                employee.Salary = input.Salary.Value;
            }

            if (!string.IsNullOrWhiteSpace(input.HireDate))
            {
                if (!DateOnly.TryParse(input.HireDate, out var newHireDate))
                    return ToolResult.Fail($"Invalid HireDate format '{input.HireDate}'. Use yyyy-MM-dd.");
                employee.HireDate = newHireDate;
            }

            if (!string.IsNullOrWhiteSpace(input.Status))
            {
                if (!AllowedStatuses.Contains(input.Status))
                    return ToolResult.Fail($"Invalid status '{input.Status}'. Allowed: Active, Inactive, On Leave.");
                employee.Status = input.Status;
            }

            await _db.SaveChangesAsync();

            // Reload navigation properties
            await _db.Entry(employee).Reference(e => e.Department).LoadAsync();
            await _db.Entry(employee).Reference(e => e.Designation).LoadAsync();

            return ToolResult.Ok(ToDto(employee), $"Employee ID {employee.EmployeeId} updated successfully.");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in UpdateEmployeeAsync");
            return ToolResult.Fail($"Database error: {ex.Message}");
        }
    }

    // -------------------------------------------------------------------------
    // Tool: get_designation_list (helper for Rust name→ID resolution)
    // -------------------------------------------------------------------------
    public async Task<ToolResult> GetDesignationListAsync()
    {
        try
        {
            var designations = await _db.Designations
                .OrderBy(g => g.DesignationId)
                .Select(g => new
                {
                    g.DesignationId,
                    g.DesignationName,
                    g.Description
                })
                .ToListAsync();

            return ToolResult.Ok(designations, $"{designations.Count} designation(s) found.");
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error in GetDesignationListAsync");
            return ToolResult.Fail($"Database error: {ex.Message}");
        }
    }
}
