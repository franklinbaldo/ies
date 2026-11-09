---
type: AgentMemory
title: "A prévia da CLI relê atribuição após POST independente no anti-IES"
description: "PR #544 prova transição SQLite por POST local seguida de prévia pelo binário real, com RED 2 falhas e GREEN CI 7/7; escrita pública ainda bloqueada."
agent: raposa
timestamp: "2026-11-09T06:31:27Z"
tags: [cli, atribuicao, anti-ies, sqlite, cassette, bdd, continuidade]
---

# Para retomar

**Fato:** [PR #544](https://github.com/franklinbaldo/ies/pull/544) Ready, head `6cdffb4`, [CI oficial #42573089845](https://github.com/franklinbaldo/ies/actions/runs/42573089845) **7/7 PASS** e [CI rápido #42573078955](https://github.com/franklinbaldo/ies/actions/runs/42573078955) PASS, `behind=0` na consulta final. Dois novos testes executam **POST independente somente local** contra anti-IES/SQLite e depois **CLI real de prévia ↔ HTTPS loopback ↔ mesmo SQLite**, observando login `aluno2` após atribuição e string vazia após remoção. São 5 requisições da CLI por prévia e nenhum POST de atribuição pela CLI; `--confirmar` público continua bloqueado.

**RED:** [CI #42572415938](https://github.com/franklinbaldo/ies/actions/runs/42572415938) 2 falhas funcionais porque a prévia lia `aluno1` antes do POST preparatório. Correção GREEN foi aplicar POST somente à fixture de teste pelo servidor existente, sem alterar produção; o harness ficou dividido em arquivo próprio, abaixo de ~16 KB. [Handoff na issue #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6733827886).

**Evidência:** sintética E2E de **estado alterado e prévia posterior PASS**; confronto independente com POST original de cassette **NOT_RUN**; treinamento/live **NOT_RUN**. A [PR #539](https://github.com/franklinbaldo/ies/pull/539), já integrada, provou menus GET (28 formulários de 7 cassettes), não efeitos POST. Não declarar completos BDD-264-02/03, nem atribuir autoria de efeito remoto a uma simples releitura.

**Cafezinho #388:** a [distinção de identidade entre duas testemunhas](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6733571131) inspira conferir estado antes e depois do POST, mas é analogia livre, não evidência do SEI.

**CONTINUAR:** 🦅 revisar, 🐘/🦡 integrar #544. Em outra fatia, completar confirmação pela CLI real e ensaio autorizado em treinamento com POST único, releitura independente, remoção, resíduo e nenhuma repetição automática.
