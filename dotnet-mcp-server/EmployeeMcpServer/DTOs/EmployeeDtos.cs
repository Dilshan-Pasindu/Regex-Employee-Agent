namespace EmployeeMcpServer.DTOs;

// ----- Employee DTOs -----

public class EmployeeDto
{
    public int EmployeeId { get; set; }
    public string FirstName { get; set; } = string.Empty;
    public string LastName { get; set; } = string.Empty;
    public string Email { get; set; } = string.Empty;
    public string? Phone { get; set; }
    public int DepartmentId { get; set; }
    public string DepartmentName { get; set; } = string.Empty;
    public int DesignationId { get; set; }
    public string DesignationName { get; set; } = string.Empty;
    public decimal Salary { get; set; }
    public string HireDate { get; set; } = string.Empty;
    public string Status { get; set; } = string.Empty;
}

public class AddEmployeeInput
{
    public string FirstName { get; set; } = string.Empty;
    public string LastName { get; set; } = string.Empty;
    public string Email { get; set; } = string.Empty;
    public string? Phone { get; set; }
    public int DepartmentId { get; set; }
    public int DesignationId { get; set; }
    public decimal Salary { get; set; }
    public string HireDate { get; set; } = string.Empty;
    public string Status { get; set; } = "Active";
}

public class UpdateEmployeeInput
{
    public int EmployeeId { get; set; }
    public string? FirstName { get; set; }
    public string? LastName { get; set; }
    public string? Email { get; set; }
    public string? Phone { get; set; }
    public int? DepartmentId { get; set; }
    public int? DesignationId { get; set; }
    public decimal? Salary { get; set; }
    public string? HireDate { get; set; }
    public string? Status { get; set; }
}

public class SearchEmployeesInput
{
    public string? FirstName { get; set; }
    public string? LastName { get; set; }
    public string? DepartmentName { get; set; }
    public string? DesignationName { get; set; }
    public string? Email { get; set; }
    public string? Status { get; set; }
}

// ----- Department DTOs -----

public class DepartmentDto
{
    public int DepartmentId { get; set; }
    public string DepartmentName { get; set; } = string.Empty;
    public string? Description { get; set; }
    public int EmployeeCount { get; set; }
}

// ----- Generic Result DTOs -----

public class ToolResult
{
    public bool Success { get; set; }
    public string? Message { get; set; }
    public object? Data { get; set; }

    public static ToolResult Ok(object? data, string? message = null) =>
        new() { Success = true, Data = data, Message = message };

    public static ToolResult Fail(string message) =>
        new() { Success = false, Message = message };
}

