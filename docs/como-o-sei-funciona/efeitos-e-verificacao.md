---
type: SEIObservation
title: Efeitos e verificação de operações
description: Leitura, escrita, recusa e efeito incerto
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
Reconcile lendo o estado, ou declare **efeito incerto**.

Exemplo: salvar o Nível de detalhe responde só com uma página que fecha
o modal. A prova é o GET seguinte da mesma tela: marcas iguais às pedidas
(sucesso), iguais às de antes (não aplicada) ou outras (efeito incerto,
se a resposta do salvar também não foi a esperada).

O acervo atual de cassettes contém as respostas, mas não
todos os corpos das requisições POST. Ele é suficiente para
estudar respostas e layouts, não para certificar por si só
uma sequência integral de escrita.
