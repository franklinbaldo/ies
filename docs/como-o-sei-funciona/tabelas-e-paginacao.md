---
type: SEIObservation
title: Tabelas e paginação
description: Cobertura, linhas, cabeçalhos e estados de listagem
---

# Tabelas e paginação

Listas de acompanhamento, blocos, textos padrão e históricos podem
paginar. Foram observadas páginas de 50 ou 100 linhas. A legenda
pode declarar um total maior do que o número de linhas presentes.
O servidor também pode lembrar a última página/filtro da sessão.

Transições da lista podem usar `infraAcaoPaginar` e
`hdnInfraPaginaAtual`. Algumas telas contêm mais de uma
listagem com paginação independente.

O IES agora compara o rótulo da unidade em `lnkInfraUnidade`, o
modo `hdnTipoVisualizacao` e os hashes de critérios
`hdnRecebidosHashCriterios`/`hdnGeradosHashCriterios`.
Esses últimos são marcadores opacos da página, não autorização,
identidade interna ou comprovante criptográfico de sessão.
Uma comparação não substitui a verificação de faixas e IDs.
Sem contexto suficiente, não juntar páginas offline.

[Relato Todos #1200](https://github.com/franklinbaldo/todos/issues/1200):
na instalação PGE-IPERON/RO, a caixa resumida paginou acima de 500
processos e tinha paginação independente de Recebidos/Gerados.
O relato recomenda testar também a visão detalhada, mas não autoriza
alterar automaticamente a preferência visual do usuário no IES.

O IES agora compara o rótulo da unidade em `lnkInfraUnidade`, o
modo `hdnTipoVisualizacao` e os hashes de critérios
`hdnRecebidosHashCriterios`/`hdnGeradosHashCriterios`.
Esses últimos são marcadores opacos da página, não autorização,
identidade interna ou comprovante criptográfico de sessão.
Uma comparação não substitui a verificação de faixas e IDs.
Sem contexto suficiente, não juntar páginas offline.

[Relato Todos #1200](https://github.com/franklinbaldo/todos/issues/1200):
na instalação PGE-IPERON/RO, a caixa resumida paginou acima de 500
processos e tinha paginação independente de Recebidos/Gerados.
O relato recomenda testar também a visão detalhada, mas não autoriza
alterar automaticamente a preferência visual do usuário no IES.

A estrutura não é uniforme: há cabeçalhos `colspan`, tabelas
que reutilizam `id` em contextos diferentes, células sem nome e
linhas de comentário adicionais. Ler por cabeçalho e identidade
comprovada é mais seguro que presumir posição absoluta.

Textos importantes podem estar em atributos `title` ou
`aria-label`: nome de unidade, usuário, tipo de processo,
anotação, sinalizações e ações disponíveis na linha.

**Nunca transformar parcial em completo:** conferir legenda,
total de registros, intervalos, número de linhas e transições.
Ausência de cursor não prova universalmente fim dos dados.
Uma listagem incompleta/indeterminada precisa informar isso.
