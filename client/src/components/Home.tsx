import { useState } from "react";

const fn = () => {
  return <div>hello</div>;
};

const Home = () => {
  const [click, setClick] = useState(0);

  return (
    <div onClick={() => console.log("clicked")}>
      <button onClick={() => setClick(click + 1)}>YAY</button>
      {fn()} {click}
    </div>
  );
};

export default Home;
