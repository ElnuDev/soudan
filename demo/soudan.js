document.getElementById("soudan").innerHTML = `<h3>Make a comment</h3>
<form id="soudan-comment-form">
	<label for="author">Name:</label> <input type="text" name="author" placeholder="Anonymous">
	<label for="email">Email:</label> <input type="email" name="email">
	<label for="text">Comment:</label>
	<textarea name="text" required></textarea>
	<input type="hidden" name="parent">
	<input type="submit">
</form>
<h3 id="soudan-comments-header">Comments</h3>
<div id="soudan-comments"></div>`;
document.write(`<script src="https://cdnjs.cloudflare.com/ajax/libs/moment.js/2.29.4/moment.min.js" crossorigin="anonymous" referrerpolicy="no-referrer"></script>`);
const url = "http://127.0.0.1:8080";
const form = document.getElementById("soudan-comment-form");
const commentContainer = document.getElementById("soudan-comments");
const commentContainerHeader = document.getElementById("soudan-comments-header");
const contentId = document.querySelector("meta[name=\"soudan-content-id\"]").getAttribute("content");
form.addEventListener("submit", e => {
	let data = {
		url: window.location.href,
		comment: { contentId }
	};
	new FormData(form).forEach((value, key) => {
		data.comment[key] = value === "" ? null : value;
	});
	fetch(url, {
		method: "POST",
		body: JSON.stringify(data),
		headers: { "Content-Type": "application/json" }
	})
		.then(response => {
			if (!response.ok) {
				return;
			}
			form.querySelector("textarea").value = "";
			reloadComments();
		})
	e.preventDefault();
});

function reloadComments() {
	fetch(`${url}/${contentId}`)
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
					html += `<div><img class="soudan-avatar" src="https://www.gravatar.com/avatar/${comment.gravatar}"><div><b>${comment.author ? comment.author : "Anonymous"}</b> commented ${moment(new Date(comment.timestamp * 1000)).fromNow()}:<br><div>${comment.text}</div></div></div>`;
				});
			}
			commentContainer.innerHTML = html;
		});
}

reloadComments();
