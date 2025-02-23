cargo build
Remove-Item ..\..\target\debug\pan.clap
Rename-Item -Path ..\..\target\debug\pan.dll -NewName pan.clap
Copy-Item -Path ..\..\target\debug\pan.clap -Destination 'C:\Program Files\Common Files\CLAP'
Copy-Item -Path ..\..\target\debug\pan.pdb -Destination 'C:\Program Files\Common Files\CLAP'