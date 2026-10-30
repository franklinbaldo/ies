---
type: Guide
title: Consulta de NUP nas caixas registradas em cassettes
description: Consulta offline verificável por NUP opaco, sem acessar o SEI
---

# Consulta de processo em uma cassette da caixa

`ies replay CENARIO --numero-processo NUP --json` consulta a identidade
observada nas tabelas de recebidos e gerados de uma cassette pública
verificada. Para cenários de paginação, fornecer os índices zero-based
das respostas necessárias com `--passo N` repetido.

Esta é uma **consulta offline de evidência**, não uma busca no SEI live
nem a leitura integral de `ies processo NUP`. Não requer sessão,
não envia GET/POST e não consulta processos fora das tabelas capturadas.

## Contrato observável

- NUP é opaco: preservar letras, pontuação e zeros à esquerda.
  Entrada vazia, controles e referências legadas `sei://` retornam
  erro tipado e saída 2, antes de interpretar a caixa.
- A consulta só pode retornar sucesso após a junção de páginas com
  identidade de unidade, filtros, modo, faixas e totais verificados,
  tanto nos recebidos quanto nos gerados. Conjunto parcial retorna
  `paginas_do_controle_nao_obtidas`, nunca resultado de ausência.
- A comparação do NUP é literal e idêntica entre tabelas. Cada
  ocorrência conserva sua categoria local, ID e número observados.
  Nenhuma correspondência significa apenas
  `numero_nao_observado_na_caixa`, **não inexistência do processo**.
- A saída JSON contém `escopo: "resumo_cassette_caixa"`,
  `instalacao` do card, `numero`, `unidade` observada e
  `ocorrencias`. Não exporta links assinados ou HTML autenticado.
- Tentar a consulta em uma cassette de histórico ou combinar a
  consulta com `--documento` falha; as operações não são
  intercambiáveis. A ausência de ZIP/card verificado falha fechada.

## TDD e validação

Testar NUP encontrado, ausente em caixa integral, caixa parcial,
faixas múltiplas, entrada inválida e operação incompatível. O mesmo
modelo de domínio é usado por `ies processo NUP --resumo`, mas o
adaptador de transporte live continua independente. Validar com
cassettes verificados de UnB, UFRJ, Goiás e PEN quando houver ZIPs
locais; não inferir compatibilidade live dessa comparação.
