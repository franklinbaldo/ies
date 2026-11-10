---
type: SEIObservation
title: Efeitos e verificação de operações
description: Leitura, escrita, recusa e escrita não confirmada
---

# Efeitos e verificação

O verbo HTTP é transporte, não prova de efeito.
`protocolo_pesquisa_rapida` foi observado como consulta por POST;
abrir telas de anotação ou tramitação pode ser GET.
Alguns GETs também podem alterar preferências de sessão.

HTTP 200 ou redirecionamento não bastam para afirmar que
uma escrita foi realizada. Uma validação rejeitada pode
retornar o mesmo formulário com mensagem de erro.

Quando possível, a prova de escrita vem da **releitura fresca**
do recurso-alvo, após o envio. A resposta imediata e o estado
observado depois são evidências diferentes.

Se ocorrer timeout depois de uma requisição mutável, o
efeito pode já ter ocorrido: não repita automaticamente.
Reconcile lendo o estado, ou declare **escrita não confirmada**.

Exemplo: salvar o Nível de detalhe responde com a mesma tela, cujo
`inicializar()` passa de `if (''=='1'){` (GET) para `if ('1'=='1'){`
seguido de `infraFecharJanelaModal()` (PEN1 5.0.4): é o sinal de que o SEI
gravou. O sinal classifica a resposta, mas a prova é o GET seguinte da
mesma tela: marcas iguais às pedidas (sucesso), iguais às de antes (não
aplicada) ou outras (escrita não confirmada, se a resposta do salvar também não
foi a esperada). Se esse GET falha, o efeito é incerto mesmo com o sinal
([diagnóstico da PEN1](../validacao-live/2026-11-04-definir-colunas-pen1.md)).

O acervo atual de cassettes contém as respostas, mas não
todos os corpos das requisições POST. Ele é suficiente para
estudar respostas e layouts, não para certificar por si só
uma sequência integral de escrita.
