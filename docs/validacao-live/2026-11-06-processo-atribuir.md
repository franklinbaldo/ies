---
type: SEIObservation
title: Validação de processo atribuir — preparação local do POST
description: Estado NOT_RUN da fatia de preparação do formulário de atribuição, sem sessão remota nem escrita, em 06/11/2026
---

# processo atribuir — 06/11/2026

PR #316, issue #264. A fatia implementa a derivação **local** do payload
observado no formulário Web e conserva a trava de segurança de
`--confirmar` antes da abertura de sessão. Testes sintéticos exercitam
a escolha pelo login exato, a remoção com `null`, a ordem dos campos
e recusas de formulário incompatível.

| Ensaio | Instalação | Estado | GET | POST | Resíduo |
| --- | --- | --- | ---: | ---: | --- |
| Prévia Web | UnB treinamento | **NOT_RUN** | 0 | 0 | nenhum |
| Prévia Web | segunda instalação de treinamento | **NOT_RUN** | 0 | 0 | nenhum |
| Confirmação única e releitura | treinamento | **NOT_RUN** | 0 | 0 | nenhum |
| Remoção única e releitura | treinamento | **NOT_RUN** | 0 | 0 | nenhum |

Não houve autenticação, requisição, escrita, retry nem consulta a produção.
A falta de ensaio live **não equivale a PASS**. A implementação continua
incompleta: faltam orçamentos exatos de rede em testes injetáveis,
reconciliação do POST com a árvore relida, replay do acervo e validação
em treinamento na ordem prévia → confirmar uma vez → verificação por leitura.
