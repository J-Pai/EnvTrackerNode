export const postEcho = async (message: string): Promise<{ echo: string }> => {
  const res = await fetch("/api/echo", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message: message }),
  });

  const res_json: { echo: string } = await res.json();
  return res_json;
};
