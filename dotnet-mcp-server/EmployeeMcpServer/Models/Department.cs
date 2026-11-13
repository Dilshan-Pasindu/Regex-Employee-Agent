using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace EmployeeMcpServer.Models;

[Table("Departments")]
public class Department
{
    [Key]
    [Column("DepartmentID")]
    public int DepartmentId { get; set; }

    [Required]
    [MaxLength(100)]
    public string DepartmentName { get; set; } = string.Empty;

    [MaxLength(255)]
    public string? Description { get; set; }

    // Navigation property
    public ICollection<Employee> Employees { get; set; } = new List<Employee>();
}

