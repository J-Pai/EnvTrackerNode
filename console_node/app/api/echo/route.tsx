export async function POST(request: Request) {
  const message_json = (await request.json()) as { message: string };

  const res = await fetch("http://localhost:8080/api/echo", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message: message_json.message }),
  });

  const res_json = await res.json();
  return Response.json(res_json);
}
