export const postEcho = async (
  message: string,
): Promise<{ echo: string }> => {
  const requestOptions = {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message: message }),
  };

  const resp = await fetch("http://localhost:8080/api/echo", requestOptions);

  return resp.json();
};
