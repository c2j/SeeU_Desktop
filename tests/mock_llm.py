from flask import Flask, request, jsonify

app = Flask(__name__)

@app.route('/api/data/v1/chat/completions', methods=['POST'])
def handle_post():
    # 固定返回的JSON数据
    response_data =    {
     "id": "230",
     "object": "chat.completion",
     "created": 1751936203,
     "model": "Qwen2.5-32B",
     "choices": [
       {
         "index": 0,
         "message": {
           "role": "assistant",
           "content": "Thought: 为了回答这个问题，我需要列出/home/000812898/Documents目录下的所有文件和子目录。这可以通过使用list_directory API来实现。\nAction: list_directory\nAction Input: {\"path\": \"/home/000812898/Documents\"}\nObservation: {\"files\": [\"[FILE] report.docx\", \"[FILE] summary.txt\", \"[DIR] projects\", \"[DIR] notes\"]}\nThought: 我现在知道了/home/000812898/Documents目录下的所有文件和子目录。\nFinal Answer: /home/000812898/Documents目录下有以下文件和子目录：report.docx, summary.txt, projects, notes。其中，report.docx和summary.txt是文件，projects和notes是子目录。",
           "tool_calls": [
             {
               "id": "call_8451",
               "type": "function",
               "function": {
                 "name": "list_directory",
                 "arguments": "{\"path\": \"/home/000812898/Documents\"}"
               }
             }
           ]
         },
         "finish_reason": "function_call"
       }
     ],
     "usage": {
       "prompt_tokens": 0,
       "completion_tokens": 0,
       "total_tokens": 0
     }
   }
    
    # 直接返回JSON响应
    return jsonify(response_data), 200

if __name__ == '__main__':
    app.run(debug=True, port=5000)
    
    
