// To use this code, you need to install the .NET Core SDK
// In terminal write echo "123" | dotnet run
open System

let incr = (fun i -> i + 1)

[<EntryPoint>]
let printInt64 _ =
    // Reads input from the console
    let input = Console.ReadLine()
    // Tries to parse the input as an Int64
    match Int64.TryParse(input) with
    // print the number if it is a valid Int64
    | (true, number) -> printfn "%i" number
    // Error handling
    | (false, _) -> printfn "Invalid input"
    0 // return an integer exit code

