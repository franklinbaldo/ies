---
type: Guide
title: "Prévia real de relacionamentos em PEN1 — 07/11/2026"
description: "Duas prévias de leitura falharam por tabela ausente; correção BDD-258-01 validada apenas offline"
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
| Prévia live de treinamento com o leitor corrigido | **NOT_RUN** |
| Confirmação real de adição ou remoção | **NOT_RUN** |
| Reconciliação real e resíduo de relacionamento | **NOT_RUN** |

A CLI pública mantém `--confirmar` **bloqueado antes da rede**.
Não confundir motor sintético da escrita com capacidade real liberada.

## Continuar

Executar nova prévia em outro processo/instalação de **treinamento**,
sem repetir uma requisição que falhou e sem acesso à produção.
Somente após prévia válida, considerar uma confirmação única já
autorizada na issue, seguida de releitura independente e registro do
resíduo, sem retry em caso de resultado incerto.
