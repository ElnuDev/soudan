const form = document.getElementById("commentForm");
const commentContainer = document.getElementById("comments");
const commentContainerHeader = document.getElementById("commentsHeader");
form.addEventListener("submit", e => {
	let data = {};
	new FormData(form).forEach((value, key) => {
		data[key] = value === "" ? null : value;
	});
	var request = new XMLHttpRequest();
	request.open("POST", "http:/127.0.0.1:8080");
	request.send(JSON.stringify(data));
	request.addEventListener("load", () => {
		form.querySelector("textarea").value = "";
		reloadComments()
	});
	request.addEventListener("error", () => alert("Comment posting failed!"));
	e.preventDefault();
});

function reloadComments() {
	fetch("http://127.0.0.1:8080")
		.then(response => {
			return response.json().then(json => {
				return response.ok ? json : Promise.reject(json);
			});
		})
		.then(comments => {
			commentContainerHeader.innerHTML = `${comments.length} Comment${comments.length == 1 ? "" : "s"}`;
			let html = "";
			if (comments.length == 0) {
				html = "<p>No comments yet! Be the first to make one.</p>";
			} else {
				comments.forEach(comment => {
					html += `<div><img class="avatar" src="https://www.gravatar.com/avatar/${comment.gravatar}"><div><b>${comment.author ? comment.author : "Anonymous"}</b> commented ${moment(new Date(comment.timestamp * 1000)).fromNow()}:<br><div>${comment.text}</div></div></div>`;
				});
			}
			commentContainer.innerHTML = html;
		});
}

reloadComments();
