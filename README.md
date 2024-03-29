# MyShare

MyShare is a convenient and secure file sharing service designed for personal use. Similar to WeTransfer, it allows you
to upload files and share links to the uploaded files.

### Key features of MyShare include:

* **Easy File Upload**: You can effortlessly upload files by simply dragging and dropping them onto the website
  interface. Whether you need to share a single file or multiple files at once, MyShare has you covered.
* **Convenient Folder Upload**: MyShare leverages the File API, enabling you to upload entire folders with their
  contents intact. This makes sharing organized sets of files a breeze.
* **Preview Before Upload**: Before initiating the upload process, MyShare offers a convenient preview of the selected
  files and folders. This allows you to verify the content you are about to share, ensuring accuracy and avoiding any
  unintended mistakes.
* **Upload Progress Feedback**: Stay informed about the progress of your file uploads with MyShare's intuitive feedback
  system. You can track the status of each upload, ensuring that your shared files are successfully transferred.
* **Efficient Files Compression**: To optimize the sharing process, MyShare automatically compresses files into a ZIP
  format. This minimizes file size while maintaining their integrity, resulting in faster and more streamlined
  transfers.
* **Admin Area with File Visualization**: Once files are uploaded, you can easily manage and visualize them in the admin
  area. This allows for convenient organization and retrieval of shared files, ensuring a smooth and efficient workflow.

With MyShare, you can enjoy a user-friendly file sharing experience that empowers collaboration while prioritizing
privacy and control.

### Authentication

* username-password file, like in [users.txt](https://github.com/NunuM/myshare-wetransfer/blob/master/users.txt) file;
* PAM authentication

### Resources

| Resource      | Method | Description                                                            |
|---------------|--------|------------------------------------------------------------------------|
| /             | GET    | index page                                                             |
| /             | POST   | upload file                                                            |
| /files        | GET    | Admin Area to see uploaded files and their correspondent sharing links |
| /share/{code} | GET    | Download file                                                          |


### Run

````bash
cargo run --color=always --package fshare --bin fshare 
````

### DEB package

````bash
cargo install cargo-deb
cargo deb
````

### Configuration

Edit to your preferences the [confi.ini](https://github.com/NunuM/myshare-wetransfer/blob/master/config.ini)
and run the application in the same directory, or you can pass the absolute path via argument or via environment 
variable: **FSHARE_CONF_FILE**


Open an issue if you find any problem 👍