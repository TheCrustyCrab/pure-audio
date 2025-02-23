cargo build
Remove-Item ..\..\target\debug\gain.clap
Rename-Item -Path ..\..\target\debug\gain.dll -NewName gain.clap
Copy-Item -Path ..\..\target\debug\gain.clap -Destination 'C:\Program Files\Common Files\CLAP'
Copy-Item -Path ..\..\target\debug\gain.pdb -Destination 'C:\Program Files\Common Files\CLAP'