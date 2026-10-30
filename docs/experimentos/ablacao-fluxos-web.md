---
type: ExperimentPlan
title: Ablação de etapas dos fluxos Web no SEI de treinamento
description: Comparações pareadas de requisições e efeitos com falha fechada
status: planned
---

# Ablação de etapas dos fluxos Web

## Questão

Quais requisições do fluxo conhecido do Todos são indispensáveis
para obter o **mesmo resultado de domínio**, com a mesma identidade,
autorização e efeitos, nas instalações SEI de treinamento?

Não usar produção nem REST/SOAP para substituir o fluxo Web. Iniciar
com o replay de cassettes existentes; não confundir captura histórica
com uma execução de ablação ao vivo.

## Desenho

Fixar instalação de treinamento, versão observada, identidade, unidade,
mesmo conjunto de processos e autorização. Guardar apenas evidências
sanitizadas, hashes e contagens, jamais cookies ou URLs assinadas.
Para cada operação, comparar referência e variantes com **uma etapa
removida por vez**; não remover duas simultaneamente antes de
identificar o efeito de cada uma.

| Operação | Ablação candidata | Critério indispensável |
| --- | --- | --- |
| Caixa da unidade | dispensar segunda página | mesmos IDs e totais recebidos/gerados |
| Processo por NUP | dispensar novo GET da caixa após leitura já disponível | mesma identidade e mesma unidade |
| Processo por NUP | dispensar abertura da árvore | mesmos campos pedidos e nenhum recebimento |
| Documento | dispensar consulta redundante antes de abrir conteúdo | número SEI, bytes e autorização iguais |
| Unidade atual | dispensar abrir menu de troca | rótulo publicado na caixa suficiente |
| Paginação | dispensar recarregar primeiro conjunto | mesmas faixas e critérios |
| Pesquisa rápida | dispensar uma releitura após redirect | não omitir verificação necessária |

A remoção não será aceita somente porque o número de requisições caiu.
Provar equivalência do resultado, dos erros e dos efeitos administrativos.
POST de paginação, pesquisa e troca precisa de exame e autorização
específicos. GET de abertura de processo também pode receber processo.

## Procedimento por instalação

1. Selecionar uma das instalações de treinamento documentadas no
   [acervo de cassettes](../cassettes.md); conferir a
   [política de acesso](../interfaces/acesso-por-instalacao.md).
2. Reproduzir a sequência de referência offline e registrar método,
   operação, dependências, quantidade de requests e dados observados.
3. Reproduzir o cenário com a etapa removida offline, quando houver
   respostas suficientes, e registrar falhas ou divergências.
4. Somente com autorização e ambiente adequado, executar a comparação
   pareada live, com identidade/unidade fixas e sem escritas em
   produção. Não disparar POST automaticamente.
5. Registrar os resultados de cada variante separadamente, inclusive
   os negativos e inconclusivos, e rever uma eventual otimização
   quando outra versão/instalação contradisser a hipótese.

## Critérios de aceitação

A ablação é aceita apenas se preservar todos os dados pedidos,
permissões, contexto, erros relevantes e efeitos. Anotar
`requests_referencia`, `requests_variante`, bytes, latência,
instalação/versão e evidência. Se uma verificação pós-escrita for
necessária, não removê-la por economia. Se falta evidência, manter
a etapa ou deixar o comando indisponível.

Os testes de [paginação](../../src/paginas.rs) estabelecem o primeiro
controle offline. Este documento **não relata testes live realizados**,
não autoriza login nem constitui certificação de compatibilidade.
