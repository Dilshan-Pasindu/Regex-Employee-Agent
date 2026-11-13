using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace EmployeeMcpServer.Models;

[Table("Designations")]
public class Designation
{
    [Key]
    [Column("DesignationID")]
    public int DesignationId { get; set; }

    [Required]
    [MaxLength(100)]
    public string DesignationName { get; set; } = string.Empty;

    [MaxLength(255)]
    public string? Description { get; set; }

    // Navigation property
    public ICollection<Employee> Employees { get; set; } = new List<Employee>();
}

