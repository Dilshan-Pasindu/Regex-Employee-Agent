using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace EmployeeMcpServer.Models;

[Table("Employees")]
public class Employee
{
    [Key]
    [Column("EmployeeID")]
    public int EmployeeId { get; set; }

    [Required]
    [MaxLength(100)]
    public string FirstName { get; set; } = string.Empty;

    [Required]
    [MaxLength(100)]
    public string LastName { get; set; } = string.Empty;

    [Required]
    [MaxLength(254)]
    public string Email { get; set; } = string.Empty;

    [MaxLength(30)]
    public string? Phone { get; set; }

    [Column("DepartmentID")]
    public int DepartmentId { get; set; }

    [Column("DesignationID")]
    public int DesignationId { get; set; }

    [Column(TypeName = "decimal(12,2)")]
    public decimal Salary { get; set; }

    [Column(TypeName = "date")]
    public DateOnly HireDate { get; set; }

    [Required]
    [MaxLength(20)]
    public string Status { get; set; } = "Active";

    // Navigation properties
    [ForeignKey("DepartmentId")]
    public Department? Department { get; set; }

    [ForeignKey("DesignationId")]
    public Designation? Designation { get; set; }
}

