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
