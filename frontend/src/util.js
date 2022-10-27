
import React, {useEffect, useState} from 'react';

const useRemoteResource = (defaultVal, requestBody, endpoint) => {
  const [count, setCount] = useState(defaultVal);
  const requestOptions = {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(requestBody)
  };
  useEffect(()=>{
    fetch('http://127.0.0.1:8080/'+endpoint,requestOptions)
      .then(response=>response.json())
      .then(data=>setCount(data))
  },[]);
  return [count,setCount];
};
export default useRemoteResource;
