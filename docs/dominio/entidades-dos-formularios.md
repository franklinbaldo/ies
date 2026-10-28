---
type: Guide
title: Entidades candidatas derivadas dos formulários SEI
description: Inventário por ciclo de vida e evidência documental sem confundir campos e entidades
---

# Entidades inferidas dos formulários

Complementa [modelo conceitual](modelo-conceitual-sei.md). O exame
foi feito sobre **cards OKF e nomes de operações capturadas**, não
sobre os corpos completos dos formulários HTML. Portanto, as entidades
abaixo são hipóteses arquiteturais com graus de prioridade, e não
atributos confirmados em todas as instalações.

## Regra de seleção

Um campo de formulário não é automaticamente entidade. Priorizar
um conceito próprio quando apresenta pelo menos um destes sinais:
identidade recuperável, ciclo de vida independente, relação
muitos-para-muitos, permissão própria ou histórico de atos.
Listas suspensas de valores podem ser vocabulários controlados,
sem exigir tabela, tipo Rust ou cliente CRUD.

## Candidatos relevantes

| Conceito | Indício no acervo | Decisão inicial |
| --- | --- | --- |
| Tipo de processo | `procedimento_escolher_tipo` | Vocabulário de domínio, com ID escopado |
| Tipo de documento | `documento_escolher_tipo` | Vocabulário de domínio |
| Hipótese legal | `hipotese_legal_select_nome_base_legal` | Referência normativa própria, não texto livre |
| Marcador | `marcador_cadastrar/alterar/desativar/reativar/excluir` | Entidade candidata forte; ciclo de vida próprio |
| Aplicação de marcador | `andamento_marcador_cadastrar/remover` | Vínculo distinto entre processo, marcador e contexto |
| Controle de prazo | `controle_prazo_definir` | Entidade/atribuição de prazo candidata; validar escopo e ciclo |
| Sobrestamento | `procedimento_sobrestar/remover_sobrestamento` | Ocorrência/estado do processo a distinguir da conclusão |
| Concessão de acesso externo | `acesso_externo_disponibilizar/cancelar` | Entidade relacional com destinatário, escopo e revogação |
| Contato/destinatário | `contato_auto_completar_acesso_externo` | Referência distinta de usuário; confirmar identidade |
| Bloco de assinatura | Operações `bloco/*` no catálogo | Entidade própria; não equivale a assinatura |
| Bloco interno | `bloco/interno/*` no OpenAPI | Entidade distinta de bloco de assinatura |
| Atribuição de processo | `procedimento_atribuicao_cadastrar` | Vínculo temporal usuário-unidade-processo |
| Anotação | `anotacao_registrar` | Registro textual próprio, distinto de andamento |
| Assunto | `GET /processo/assunto/pesquisar` | Vocabulário classificatório |
| Credencial de acesso | `processo/{id}/credenciamento/*` REST | Vínculo autorizativo independente, com revogação |
| Acompanhamento especial | `acompanhamentoespecial/listar` REST | Vínculo de acompanhamento, não tramitação |

## Distinções que evitam erro de domínio

- **Marcador** (catálogo) não é a aplicação do marcador a um
  processo; esta última pode ter histórico, anotação ou contexto.
- **Prazo** não é simplesmente a data de criação do processo;
  verificar unidade, responsável, início, término e alteração.
- **Sobrestamento** não é conclusão de atuação na unidade.
- **Assinatura** (ato) não é **bloco de assinatura** (organização e
  disponibilização de documentos).
- **Interessado**, **contato**, **usuário externo** e **destinatário
  de concessão** não são automaticamente a mesma entidade.
- **Anotação** e **andamento descritivo** não devem ser fundidos
  apenas porque possuem texto livre.
- **Tipo de documento**, **hipótese legal**, **assunto** e **nível
  de acesso** podem ser referências controladas, sem CRUD no IES.

## Evidências indicativas

- [Cadastro e listas — Goiás](../../cassettes/go/cadastro-listas-go--beb707305040.md)
- [Marcadores — Goiás](../../cassettes/go/marcador-escritas-go--7c45b39d550f.md)
- [Controle de prazo — Goiás](../../cassettes/go/processo-prazo-go--4d9b8462dae9.md)
- [Sobrestamento — Goiás](../../cassettes/go/processo-sobrestar-retomar-go--4fb6c8fe18df.md)
- [Acesso externo — Goiás](../../cassettes/go/processo-acesso-externo-go--34dedaf86d52.md)
- [Catálogo de endpoints REST](../interfaces/rest-endpoints.md)

Os cards documentam ações HTTP gravadas e não o resultado final de
cada formulário, os campos exatos ou a semântica de todas as versões.
A disponibilidade REST por instalação permanece desconhecida salvo
teste autorizado.

## Próximos ensaios

1. Abrir os formulários reais gravados de **marcador** e de
   **acesso externo**; listar campos e distinguir catálogo, vínculo
   e eventos de alteração/revogação.
2. Investigar **controle de prazo** e **sobrestamento**: identificar
   estados, datas, competência da unidade e reversões.
3. Comparar **tipo de processo**, **tipo de documento**, **assunto**
   e **hipótese legal** entre duas instalações: IDs, nomes e escopo.
4. Investigar **bloco de assinatura** e **bloco interno** antes de
   criar uma generalização `Bloco`.
5. Só promover conceitos a structs Rust com comando, contrato,
   campos verificáveis e teste reproduzível.

**Fora do escopo deste avanço:** download ou leitura dos ZIPs,
execução live em SEI, criação de schema relacional e alterações do
JSON público ou do runtime.
