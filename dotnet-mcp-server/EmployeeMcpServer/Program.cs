// =============================================================================
// Employee MCP Server — Program.cs
// MCP STDIO transport server using the official ModelContextProtocol SDK.
//
// CRITICAL: STDOUT is reserved exclusively for MCP JSON-RPC protocol messages.
//           All logs and diagnostics must write to STDERR only.
// =============================================================================

using EmployeeMcpServer.Data;
using EmployeeMcpServer.Services;
using EmployeeMcpServer.Tools;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

// Redirect all console output to STDERR to protect the MCP STDIO protocol stream
Console.SetOut(Console.Error);

var builder = Host.CreateApplicationBuilder(args);

// Ensure appsettings.json is loaded from the binary directory and environment variables
builder.Configuration
    .SetBasePath(AppContext.BaseDirectory)
    .AddJsonFile("appsettings.json", optional: true, reloadOnChange: false)
    .AddEnvironmentVariables();

// ---- Logging: write ONLY to STDERR, never STDOUT ----
builder.Logging.ClearProviders();
builder.Logging.AddConsole(options =>
{
    options.LogToStandardErrorThreshold = Microsoft.Extensions.Logging.LogLevel.Trace;
});
builder.Logging.SetMinimumLevel(LogLevel.Warning);

// ---- Database: EF Core + Pomelo MySQL ----
var connectionString = builder.Configuration["ConnectionStrings:DefaultConnection"]
    ?? throw new InvalidOperationException(
        "Connection string 'DefaultConnection' not found. " +
        "Set it in appsettings.json or via CONNECTIONSTRINGS__DEFAULTCONNECTION environment variable.");

builder.Services.AddDbContext<EmployeeDbContext>(options =>
    options.UseMySql(
        connectionString,
        ServerVersion.AutoDetect(connectionString),
        mySqlOptions => mySqlOptions.EnableRetryOnFailure(3)
    )
);

// ---- Application services ----
builder.Services.AddScoped<EmployeeService>();

// ---- MCP Server: STDIO transport ----
builder.Services
    .AddMcpServer()
    .WithStdioServerTransport()
    .WithToolsFromAssembly();

var host = builder.Build();

// ---- Verify database connectivity before serving MCP requests ----
using (var scope = host.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<EmployeeDbContext>();
    try
    {
        await db.Database.CanConnectAsync();
        Console.Error.WriteLine("[MCP Server] Database connection verified.");
    }
    catch (Exception ex)
    {
        Console.Error.WriteLine($"[MCP Server] FATAL: Cannot connect to database: {ex.Message}");
        Console.Error.WriteLine("[MCP Server] Check your connection string in appsettings.json.");
        Environment.Exit(1);
    }
}

Console.Error.WriteLine("[MCP Server] Employee MCP Server starting on STDIO transport...");

await host.RunAsync();
