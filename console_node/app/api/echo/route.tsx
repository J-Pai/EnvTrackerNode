export async function POST(request: Request) {
  const message_json = (await request.json()) as { message: string };

  const res = await fetch("https://localhost:8443/api/echo", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message: message_json.message }),
  });

  const res_json = await res.json();
  return Response.json(res_json);
}
