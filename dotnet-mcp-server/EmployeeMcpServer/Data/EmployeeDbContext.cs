using EmployeeMcpServer.Models;
using Microsoft.EntityFrameworkCore;

namespace EmployeeMcpServer.Data;

public class EmployeeDbContext : DbContext
{
    public EmployeeDbContext(DbContextOptions<EmployeeDbContext> options) : base(options) { }

    public DbSet<Department> Departments { get; set; }
    public DbSet<Designation> Designations { get; set; }
    public DbSet<Employee> Employees { get; set; }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);

        // Department
        modelBuilder.Entity<Department>(entity =>
        {
            entity.ToTable("Departments");
            entity.HasKey(d => d.DepartmentId);
            entity.Property(d => d.DepartmentId).HasColumnName("DepartmentID");
            entity.Property(d => d.DepartmentName).IsRequired().HasMaxLength(100);
            entity.Property(d => d.Description).HasMaxLength(255);
            entity.HasIndex(d => d.DepartmentName).IsUnique();
        });

        // Designation
        modelBuilder.Entity<Designation>(entity =>
        {
            entity.ToTable("Designations");
            entity.HasKey(d => d.DesignationId);
            entity.Property(d => d.DesignationId).HasColumnName("DesignationID");
            entity.Property(d => d.DesignationName).IsRequired().HasMaxLength(100);
            entity.Property(d => d.Description).HasMaxLength(255);
            entity.HasIndex(d => d.DesignationName).IsUnique();
        });

        // Employee
        modelBuilder.Entity<Employee>(entity =>
        {
            entity.ToTable("Employees");
            entity.HasKey(e => e.EmployeeId);
            entity.Property(e => e.EmployeeId).HasColumnName("EmployeeID");
            entity.Property(e => e.FirstName).IsRequired().HasMaxLength(100);
            entity.Property(e => e.LastName).IsRequired().HasMaxLength(100);
            entity.Property(e => e.Email).IsRequired().HasMaxLength(254);
            entity.Property(e => e.Phone).HasMaxLength(30);
            entity.Property(e => e.DepartmentId).HasColumnName("DepartmentID");
            entity.Property(e => e.DesignationId).HasColumnName("DesignationID");
            entity.Property(e => e.Salary).HasColumnType("decimal(12,2)");
            entity.Property(e => e.Status).IsRequired().HasMaxLength(20).HasDefaultValue("Active");

            entity.HasIndex(e => e.Email).IsUnique();
            entity.HasIndex(e => e.DepartmentId);
            entity.HasIndex(e => e.DesignationId);
            entity.HasIndex(e => e.Status);

            // Foreign keys
            entity.HasOne(e => e.Department)
                  .WithMany(d => d.Employees)
                  .HasForeignKey(e => e.DepartmentId)
                  .OnDelete(DeleteBehavior.Restrict);

            entity.HasOne(e => e.Designation)
                  .WithMany(d => d.Employees)
                  .HasForeignKey(e => e.DesignationId)
                  .OnDelete(DeleteBehavior.Restrict);
        });
    }
}

