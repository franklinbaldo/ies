---
type: AgentMemory
title: "Prévia de processo.criar no anti-IES exige menu observado e nenhum POST"
description: "PR #518: BDD-245-01 testado pela CLI real em HTTPS loopback, SQLite v16 e 3 GET, sem criação; RED oficial e GREEN 7/7."
agent: coruja
timestamp: "2026-11-08T23:42:32Z"
tags: [processos, anti-ies, bdd, criacao, sqlite, cassettes, reconciliacao]
---

# Retomada

**Fato:** a [PR #518](https://github.com/franklinbaldo/ies/pull/518), vinculada à [issue #245](https://github.com/franklinbaldo/ies/issues/245), implementa a prévia do **BDD-245-01** por CLI real, HTTPS local e anti-IES usando **um SQLite efêmero compartilhado**. A migração 0016 registra tipo e assunto do processo; as telas de menu, escolha e cadastro derivam dessa relação. O E2E exige **3 GET e 0 POST**, testa **0 processos antes e depois** no banco e verifica `confirmado=false` e NUP ausente. A confirmação de criação permanece recusada; nenhum SEI real foi acessado.

**RED observado:** [CI #42535232315](https://github.com/franklinbaldo/ies/actions/runs/42535232315) reprovou o E2E por `link_de_iniciar_processo_ausente`. A correção constrói o menu somente para tipos explicitamente registrados: **ausência de observação não é lista vazia observada**. A migração elevou `PRAGMA user_version` a 16, exigindo revisão das expectativas anteriores no teste de documentos e nos testes de controle e retornos. Após essas correções, [CI #42536191661](https://github.com/franklinbaldo/ies/actions/runs/42536191661) ficou **7/7** e [CI rápido #42536178596](https://github.com/franklinbaldo/ies/actions/runs/42536178596) verde no head `9766a66f`.

**Cassettes:** [inspeção Actions #42535227162](https://github.com/franklinbaldo/ies/actions/runs/42535227162) validou **2 cards UnB / 18 interações**, hashes SHA-1/SHA-256 e estrutura. O arquivo temporário de inspeção foi removido. O replay existente do leitor passou no CI, porém **não houve confronto semântico independente da resposta do anti-IES com esses arquivos**. Treinamento/live e REA: **NOT_RUN**. Os tipos usados no E2E são dados sintéticos revisáveis e não alegação de observação direta da UnB.

**Retomar:** 🦅 revisar e 🐘/🦡 integrar #518; depois implementar **BDD-245-02** (uma criação e releitura causal) e **BDD-245-03** (efeito incerto sem retry). Nunca inferir autoria de NUP apenas pelo resultado da caixa Gerados, nem habilitar escrita pública a partir do ensaio. [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6728728915) recebeu a distinção entre falta de observação e vazio observado.
