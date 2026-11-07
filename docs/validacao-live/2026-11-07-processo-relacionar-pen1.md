---
type: Guide
title: "Prévia real de relacionamentos em PEN1 — 07/11/2026"
description: "Duas prévias falharam por tabela ausente; terceira prévia PEN1 passou após a correção"
---

# Processo relacionar — PEN1 em 07/11/2026

Issue [#258](https://github.com/franklinbaldo/ies/issues/258) ·
[PR #390](https://github.com/franklinbaldo/ies/pull/390).

## Observação live antes da correção

A instalação **PEN1 de treinamento**, com credenciais públicas, foi
consultada em **duas execuções distintas**. O roteiro de cada ensaio
realizou login, leitura da lista e uma prévia de
`ies processo relacionar NUP --com DESTINO`, sem `--confirmar`.

| Etapa | Ensaio 19:14 UTC | Ensaio 19:21 UTC |
| --- | --- | --- |
| Login | PASS, exit 0, 6 requisições | PASS, exit 0, 6 requisições |
| Listar processos | PASS, exit 0, 100 NUPs distintos | PASS, exit 0, 100 NUPs distintos |
| Prévia de relacionamento | **FAIL**, exit 1 | **FAIL**, exit 1 |
| Motivo | `tabela_de_relacionamentos_ausente_ou_repetida` | Mesmo motivo; diagnóstico estrutural: 1 formulário, 0 tabelas |
| Escrita modificadora | **0** | **0** |
| Repetições/retry | **0** | **0** |

Fontes: [primeiro ensaio](https://github.com/franklinbaldo/ies/issues/258#issuecomment-6711858997)
e [segundo ensaio](https://github.com/franklinbaldo/ies/issues/258#issuecomment-6711931237).

**Orçamento observado:** cada login declarou 6 requisições.
A contagem das chamadas de listar e da prévia não foi publicada no
resultado; portanto, **não há total HTTP exato comprovado**.
O GET da abertura do processo pode registrar recebimento no SEI;
não se afirma ausência de todo efeito colateral de leitura.
Nenhum POST de modificação de relacionamento foi tentado, e nenhum
vínculo novo foi declarado ou comprovado.

## BDD-258-01: correção e validação sintética

O leitor antes exigia uma tabela com
`summary="Lista de Processos Relacionados"`. A nova fatia admite
**lista vazia sem tabela** apenas no formulário
`frmProcedimentoListarRelacionamentos` com exatamente um campo
`txtProtocolo` e um `hdnIdProtocolo`. O cenário sintético
usa abertura de processo, formulário sem tabela e complemento XML:
**4 GET + 1 POST de consulta, 0 POST de modificação**.

- [RED isolado 42406387012](https://github.com/franklinbaldo/ies/actions/runs/42406387012):
  1 FAIL, 0 PASS, antes da correção.
- [GREEN isolado 42406513028](https://github.com/franklinbaldo/ies/actions/runs/42406513028):
  1 PASS, 0 FAIL. O workflow completo falhou por Rustfmt e checkout raso,
  corrigidos posteriormente; não confundir esse job com o CI oficial.
- [CI oficial no head 76d7576](https://github.com/franklinbaldo/ies/actions/runs/42407254378):
  7/7 PASS. [Wheel](https://github.com/franklinbaldo/ies/actions/runs/42407254405):
  3/3 PASS.

| Validação após a correção | Estado |
| --- | --- |
| BDD sintético da prévia sem tabela | **PASS** |
| Prévia live de treinamento com o leitor corrigido | **PASS em PEN1** |
| Confirmação real de adição ou remoção | **NOT_RUN** |
| Reconciliação real e resíduo de relacionamento | **NOT_RUN** |

A CLI pública mantém `--confirmar` **bloqueado antes da rede**.
Não confundir motor sintético da escrita com capacidade real liberada.

## Revalidação live após a correção

Em **07/11/2026 às 20:13:07 UTC**, outro par de processos da **PEN1 de
treinamento** (não os quatro números das tentativas anteriores) foi
usado com `ies processo relacionar NUP --com DESTINO`, sem `--confirmar`.
A [evidência saneada registrada na issue](https://github.com/franklinbaldo/ies/issues/258#issuecomment-6712482131)
mostra:

| Etapa | Resultado observado |
| --- | --- |
| Login | **PASS**, exit 0, 6 requisições |
| Listar processos | **PASS**, exit 0, 100 processos distintos na página |
| Prévia da relação | **PASS**, exit 0, 4 requisições reportadas pela CLI |
| Indicadores da prévia | `sem_efeito=false`, `confirmado=false`, `alterado=false` |
| Escrita modificadora / retry | **0 / 0** |

A contagem `4` pertence apenas ao comando de prévia; não é o total
de requests do login, listagem e prévia, nem discrimina métodos HTTP.
O ensaio **comprova a prévia BDD-258-01 na PEN1**, não valida
confirmação de adição/remoção, reconciliação ou compatibilidade na UnB.
O GET de abertura pode registrar recebimento; não se declara ausência
de efeitos colaterais de leitura.

## Continuar

A prévia PEN1 passou. Para BDD-258-02/03/04, habilitar confirmação
apenas no ensaio controlado em treinamento autorizado, com **um único
POST mutador**, releitura independente e registro do resíduo. Não
repetir escrita de resultado incerto. A CLI pública permanece bloqueada
até validar esse caminho; UnB ainda não foi testada.
