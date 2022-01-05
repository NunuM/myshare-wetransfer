const filesToSend = new Map();

function selectedFiles() {
    let files = document.getElementById("file").files;

    for (let file of files) {
        if (!filesToSend.has(file.name)) {
            addFileLine(file);
            filesToSend.set(file.name, file);
        }
    }
}

function selectedFolder() {
    let files = document.getElementById("folder").files;

    for (let file of files) {
        let key = file.webkitRelativePath ? file.webkitRelativePath : file.name;
        if (!filesToSend.has(key)) {
            addFileLine(file, file.webkitRelativePath);
            filesToSend.set(key, file);
        }
    }
}

function addFileLine(file, fullPath = null) {

    const li = document.createElement("li");
    li.classList.add('list-group-item', 'animate__animated', 'animate__fadeInRight');


    li.innerHTML = `
<div style="display: flex; justify-content: space-between;">
    <div>
        <p>${fullPath ? fullPath : file.name}<br>
            <small>${humanFileSize(file.size)}</small>
        </p>
    </div>
    <div>
      <button onclick="deleteFile('${fullPath ? fullPath : file.name}', this)" type="button" class="btn-close" aria-label="Close"></button>
    </div>
</div>`

    document.getElementById('list').appendChild(li);

}

function deleteFile(filename, source) {
    filesToSend.delete(filename);
    const div = source.parentElement.parentElement.parentElement;

    div.classList.remove('animate__fadeInRight')
    div.classList.add('animate__fadeOut')

    setTimeout(() => {
        source.parentElement.parentElement.parentElement.remove();
    }, 800);
}

function humanFileSize(size) {
    if (size === 0) {
        return '0 B';
    }
    let i = Math.floor(Math.log(size) / Math.log(1000));
    return (size / Math.pow(1000, i)).toFixed(2) * 1 + ' ' + ['B', 'kB', 'MB', 'GB', 'TB'][i];
}

function send() {
    const payload = new FormData();

    for (const [name, file] of filesToSend.entries()) {
        payload.append(name, file);
    }

    let addBtn = document.getElementById('add-file');
    addBtn.style.pointerEvents = 'none';

    document.querySelectorAll("button").forEach(b => {
        b.setAttribute('disabled', '');
    });

    document.getElementById('link-spinner').style.visibility = '';

    const ajax = new XMLHttpRequest();

    ajax.upload.addEventListener("progress", progressHandler, false);
    ajax.addEventListener("load", completeHandler, false);
    ajax.addEventListener("error", errorHandler, false);
    ajax.addEventListener("abort", abortHandler, false);

    ajax.open("POST", "/");
    ajax.send(payload);
}


function progressHandler(ev) {
    let percent = Math.trunc((ev.loaded / ev.total) * 100);

    let progress = document.getElementById("link-indicator");

    progress.setAttribute("aria-valuenow", percent.toString());
    progress.style.width = progress.textContent = `${percent}%`;
}

function completeHandler(ev) {

    if (ev.target.status !== 200) {
        errorHandler(ev);
        return;
    }

    const btn = document.getElementById('link');

    btn.classList.remove('btn-dark');
    btn.classList.add('btn-success');
    btn.removeAttribute('disabled');

    document.getElementById('link-indicator').innerText = 'File Uploaded'
}

function errorHandler(ev) {
    console.log(ev);

    const btn = document.getElementById('link');

    btn.classList.remove('btn-dark');
    btn.classList.add('btn-danger');

    btn.innerText = "Error Uploading: " + ev.target.responseText;
}

function abortHandler(ev) {
    alert("Abort")
}

function openFiles() {
    document.getElementById("file").click();
}

function openFolder() {
    document.getElementById("folder").click();
}

function dropHandler(ev) {
    ev.preventDefault();

    if (ev.dataTransfer.items) {
        getAllFileEntries(ev.dataTransfer.items).then((files) => {
            files.forEach((file) => {
                file.file((asFile) => {
                    filesToSend.set(asFile.webkitRelativePath, asFile);
                    addFileLine(asFile, asFile.webkitRelativePath)
                });
            });
        }).catch((error) => {
            console.log(error);
        });
    } else {
        for (let i = 0; i < ev.dataTransfer.files.length; i++) {
            let file = ev.dataTransfer.files[i];
            addFileLine(file);
            filesToSend.set(file.name, file);
        }
    }
}


function dragOverHandler(ev) {
    ev.preventDefault();
}

async function getAllFileEntries(dataTransferItemList) {
    let fileEntries = [];

    let queue = [];

    for (let i = 0; i < dataTransferItemList.length; i++) {
        queue.push(dataTransferItemList[i].webkitGetAsEntry());
    }
    while (queue.length > 0) {
        let entry = queue.shift();
        if (entry.isFile) {
            fileEntries.push(entry);
        } else if (entry.isDirectory) {
            queue.push(...await readAllDirectoryEntries(entry.createReader()));
        }
    }
    return fileEntries;
}

async function readAllDirectoryEntries(directoryReader) {
    let entries = [];
    let readEntries = await readEntriesPromise(directoryReader);
    while (readEntries.length > 0) {
        entries.push(...readEntries);
        readEntries = await readEntriesPromise(directoryReader);
    }
    return entries;
}

async function readEntriesPromise(directoryReader) {
    try {
        return await new Promise((resolve, reject) => {
            directoryReader.readEntries(resolve, reject);
        });
    } catch (err) {
        console.log(err);
    }
}
