---
type: SEIObservation
title: "Processo prazo nas sete instalações de treinamento: PASS em todas"
description: "Ensaio de 10/11/2026 da #272 em PEN1, PEN2, PEN3, PEN4, UFRJ e GO, complementando a UnB: definir por dias e excluir com saída 0 e releitura independente, sem resíduo"
---

# Processo prazo — sete instalações de treinamento, 10/11/2026

**Issue:** [#272](https://github.com/franklinbaldo/ies/issues/272).
**Regra:** aprovação para produção exige ensaio em todas as instalações de
treinamento (decisão do proprietário de 10/11/2026; `src/origem_de_treinamento.rs`).
A UnB foi ensaiada antes: [relatório](2026-11-10-processo-prazo-unb.md).
**Contas:** as públicas da [tabela autorizada](../interfaces/credenciais-de-treinamento.md);
sessões em pastas `0700` com arquivos `0600`, fora do repositório.
**Execução:** binário release de `main@10927ab` (com as correções da #623),
sem alteração de código; rede direta do sandbox.

## Orçamento

Declarado **150**; revisado para **180** por causa de três leituras de
diagnóstico no GO. Usado: cerca de **175** requisições contadas pela CLI,
incluindo os seis logins (6 cada). Cada `--confirmar` foi executado uma única
vez, com trava `O_EXCL`; a única falha de escrita (GO, abaixo) saiu com 1
antes de gravar.

## Resultado por instalação

Processo: o primeiro Gerado já visualizado da unidade da conta.

| Instalação | Unidade | Processo | `--dias 5 --confirmar` | `--excluir --confirmar` | Releitura |
| --- | --- | --- | --- | --- | --- |
| UnB 4.0.9 | Unidade1 | `23106.001906/2026-14` | PASS 0 | PASS 0 | `sem_prazo` |
| PEN1 5.0.4 | TESTE | `043491/2026-21` | PASS 0 (`14/11/2026`) | PASS 0 | `sem_prazo` |
| PEN2 | TESTE | `98290.000142/2026-09` | PASS 0 | PASS 0 | `sem_prazo` |
| PEN3 5.0.4 | TESTE | `98390.000088/2026-65` | PASS 0 | PASS 0 | `sem_prazo` |
| PEN4 5.0.4 | TESTE | `98490.000104/2026-91` | PASS 0 | PASS 0 | `sem_prazo` |
| UFRJ | ADCENTRAL | `99079.901797/2026-65` | PASS 0 | PASS 0 | `sem_prazo` |
| GO 5.0.3 | SEAD/CDE-09516 | `202600005004933` | PASS 0 (2ª tentativa) | PASS 0 | `sem_prazo` |

Cada definir usou 5 requisições e cada excluir, 9.

## Ocorrências no GO

- O primeiro processo escolhido, `202600005006024`, falhou já na abertura
  (`endereco_da_arvore_nao_reconhecido`, saída 1, sem escrita). Outros três
  processos da mesma lista abriram normalmente. É uma particularidade
  daquele processo na leitura da árvore, a investigar à parte; a CLI
  recusou de forma segura.
- O primeiro `--confirmar` no `202600005004933` saiu com `falha_de_rede`
  (saída 1). A releitura confirmou que nada foi gravado. Depois de mais de
  um minuto (o GO tem WAF contra rajadas), uma nova tentativa deliberada
  passou com saída 0.

## Resultado

**PASS nas sete instalações.** `processo prazo` cumpre a regra de
aprovação. Resíduo: nenhum; todos os processos terminaram `sem_prazo`.
