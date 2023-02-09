pub static ADDON_JSON: &str = r#"
{
	"title": "%NAME%",
	"type":	"%TYPE%",
	"tags":	[ %TAGS% ],
	"ignore":
	[
		"*.psd",
		"*.vcproj",
		"*.svn*",
		"*.db",
		"thumbs.db",
		"Thumbs.db",
		"*.ini",
		"desktop.ini",
		"Desktop.ini",
		"*.log",
		"*.prt",
		"*.vmf",
		"*.vmx",
		"*.bat",
		"*.txt"
	]
}
"#;