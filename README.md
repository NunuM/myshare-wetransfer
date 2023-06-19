# MyShare

MyShare is a service like WeTransfer, but for personal use. In WeTransfer, they hold your files, in this
service, you share the URL with your colleges and they upload the files. You can also share links like WeTransfer
from files that you uploaded.

### Features

* Drag-and-drop;
* Single and multiple files uploads;
* Folder upload (using File API);
* Preview of files and folder before upload;
* Upload progress feedback;
* Files compression (zip);
* Visualization of uploaded files in admin area;

### Environment Variables

**FS_UPLOAD_DIR**

Absolute path from where you want store the files.

**FS_TEMPLATES**

Absolute path for application HTML tera templates.

**FS_PORT**

Port to start the server.

**FS_MAX_SIZE**

Max allowed file size;

**FS_AUTH**

Base64 encoded string for Basic Authentication to access to: /files
