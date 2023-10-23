export const postEcho = async (
  message: string,
): Promise<{ echo: string }> => {
  const resp = await fetch("http://localhost:8080/api/echo", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message: message }),
    mode: "no-cors",
  });

  console.log(resp);

  return { echo: "Hello World" };
};
