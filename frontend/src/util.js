
import {useEffect, useState} from 'react';

function wait(delay){
    return new Promise((resolve) => setTimeout(resolve, delay));
}

function fetchRetry(url, delay, tries, fetchOptions = {}) {
    function onError(err){
        let triesLeft = tries - 1;
        if(!triesLeft){
            throw err;
        }
        return wait(delay).then(() => fetchRetry(url, delay, triesLeft, fetchOptions));
    }
    return fetch(url,fetchOptions).catch(onError);
}
export const useRemoteResource = (defaultVal, requestBody, endpoint, effectHook=[]) => {
  const [count, setCount] = useState(defaultVal);
  const requestOptions = {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(requestBody)
  };
  useEffect(()=>{
    fetchRetry('http://127.0.0.1:12000/'+endpoint,200,10000,requestOptions)
      .then(response=>response.json())
      .then(data=>setCount(data))
  },effectHook);
  return [count,setCount];
};
export const callRemote = (requestBody, endpoint) => {
  const requestOptions = {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(requestBody)
  };
  return fetch('http://127.0.0.1:12000/'+endpoint,requestOptions)
};
