---
type: SEIObservation
title: "UnB treinamento: prévia válida, confirmação única e efeito incerto"
description: "CLI Rust real observou estado posterior aluno2 após confirmação única, mas devolveu saída 3 sem prova causal; remoção não executada e possível resíduo preservado."
---

# Processo atribuir — confirmação única inconclusiva na UnB (09/11/2026)

**Escopo:** [issue #264](https://github.com/franklinbaldo/ies/issues/264), BDD-264-01/02/03 canônicos. Binário Rust release do [CI oficial #42692335479](https://github.com/franklinbaldo/ies/actions/runs/42692335479), commit `cb5a62224b89e883aee2976d501d85b70fc2c863`, artefato `ies-linux-x86_64-cb5a62224b89e883aee2976d501d85b70fc2c863`. SHA-256 do executável comparado ao `ies.sha256`: **PASS**.

**Ambiente:** execução controlada na ponte Jatobá, **não no GitHub Actions**; UnB **exclusivamente de treinamento**, origem conferida por igualdade com a tabela pública do repositório, arquivos de sessão/credenciais privados `0600`, diretório privado `0700`. Nenhum valor de senha, cookie, URL assinada ou `infra_hash` foi publicado. Os comandos preservaram intervalos de pelo menos 30 segundos e não repetiram uma operação falha. O GET de um processo pode ter efeitos administrativos de leitura.

## Observação por etapas

| Etapa da CLI real | Estado | Requisições reportadas |
| --- | --- | ---: |
| `ies --json login` | PASS, `login_realizado` | 6 |
| `ies unidade recebidos` | PASS, primeira página com 21 processos | `null` (não observadas) |
| `ies processo NUP`, com filtros de componentes | PASS, identidade e unidade verificadas, ausência de atribuição observada | 2 |
| Prévia `ies processo atribuir NUP --usuario aluno2` | **PASS**, `confirmado=false`, `alterado=false`, nenhum POST | 3 |
| Nova prévia dentro do executor manual, antes de confirmar | **PASS**; situação atual explicitamente vazia | 3 |
| `ies processo atribuir NUP --usuario aluno2 --confirmar` | **EFEITO_INCERTO, saída 3** | 5 |
| Verificação adicional e remoção | **NOT_RUN**, por efeito incerto | 0 chamadas adicionais |

**NUP usado:** `23106.001037/2025-39`, descoberto e verificado nesta execução, diferente do candidato da rodada anterior. O login exato `aluno2` foi aceito na prévia atual.

O executor autorizado `scripts/ensaio_processo_atribuir.py` invocou **uma única confirmação** após sua própria prévia válida. A CLI terminou com **saída 3**, e o resumo registrou `atribuicao_relida: "aluno2"`. Isto significa que uma leitura posterior **observou** o estado solicitado, **não** que a mudança tenha sido causada por este POST. Não é possível afirmar a partir do registro saneado qual aspecto da resposta impediu a comprovação causal. Não houve retry nem uma segunda confirmação.

O executor terminou `EFEITO_INCERTO`, com `residuo: possivel_atribuicao`, mantendo o registro privado `0600` sob nome exclusivo no ambiente de execução. Não fez prévia de remoção nem `--remover --confirmar`, porque a confirmação anterior não pôde ser comprovada. **Pode ter restado uma atribuição a `aluno2`** no ambiente compartilhado de treinamento. O estado futuro pode mudar independentemente desta sessão.

## Três níveis de evidência

- **IES ↔ anti-IES ↔ SQLite:** compatibilidade sintética **PASS em fatias anteriores**, **NOT_RUN neste binário/ensaio**. Esta observação externa não converte o PASS antigo em prova nova.
- **IES e anti-IES ↔ cassettes:** confronto histórico dos **28 formulários GET** e caracterização das **14 respostas POST** anteriores; **NOT_RUN aqui** para replay causal de uma escrita. Os 302 históricos não demonstram, por si, autoria da alteração observada.
- **Treinamento:** login, descoberta e duas prévias **PASS**. Confirmação única **INCONCLUSIVA (saída 3)**; remoção **NOT_RUN**. Não há confirmação causal da atribuição nem comprovação de resíduo limpo.

**Próxima ação:** preservar o registro da tentativa, não repetir a atribuição. Investigar as provas já capturadas e, se decidir tratar o possível resíduo, fazer isso como operação distinta e consciente, precedida por nova prévia e nova leitura do estado atual; nunca chamar uma remoção de confirmação retroativa da autoria do POST anterior. O contrato público deve continuar recusando declarar sucesso sem identidade causal.
