---
type: Guide
title: Validação dos vínculos centrais do domínio SEI
description: Evidências e testes discriminantes para tramitação, identidade e assinatura
---

# Vínculos centrais: evidência e implementação

Complementa [o modelo conceitual](modelo-conceitual-sei.md).
As cassettes citadas são **cards OKF** que apontam para capturas
originais no Internet Archive. A presença de uma rota no card ou no
OpenAPI **não prova** execução com sucesso nem semântica equivalente.
Não introduzir novos tipos Rust só por constarem deste inventário.

## Processo e unidade: investigação da tramitação

O [card Goiás, processo-escritas](../../cassettes/go/processo-escritas-go--841b0341a184.md)
aponta para 40 interações HTTP, com:
- `procedimento_enviar`: 3;
- `procedimento_concluir`: 2;
- `procedimento_reabrir`: 1;
- `procedimento_consultar_historico`: 1;
- `unidade_auto_completar_envio_processo`: 2.

Há cards equivalentes de [UnB](../../cassettes/unb/processo-escritas-unb--47c0acc994ac.md),
[UFRJ](../../cassettes/ufrj/processo-escritas-ufrj--d21a20db9b10.md)
e PEN1–PEN4. Este levantamento verifica **presença de nomes de
operações no índice**, não payloads, identidades nem efeitos.

O [índice REST oficial](../interfaces/rest-endpoints.md) distingue
`POST /processo/enviar`, `POST /processo/receber`,
`POST /processo/concluir` e `POST /processo/reabrir/{procedimento}`.
Isso comprova que o contrato publicado distingue verbos; **não**
comprova as transições de estado do cliente Web nem sua disponibilidade
em GO/UnB/UFRJ/PEN.

### Questões discriminantes

| Pergunta | Comparação necessária | Consequência |
| --- | --- | --- |
| Um envio agrupa múltiplos destinatários? | Payload e confirmação da mesma operação | Não usar destinatário singular sem prova |
| Enviar encerra a atuação remetente? | Estado antes/depois, opção manter aberto | Não inferir conclusão de um envio isolado |
| Receber tem identidade própria? | IDs e andamentos antes/depois | Evitar confundir remessa e recebimento |
| Reabrir necessita novo envio? | Histórico e tela após reabertura | Separar atuação local de trânsito |
| Há ID de remessa estável? | GETs distintos e histórico | Não criar `IdTramitacao` antecipadamente |
| Sigiloso segue mesmas regras? | Manual e ambiente controlado autorizado | Não generalizar fluxo ordinário |

**Decisão provisória:** `AtuacaoProcessoUnidade` é conceito
contextual; `Envio`, `Recebimento` e `Andamento` são candidatos
distintos. Nenhum exige struct persistida antes de um contrato de
consulta e replay que discrimine os efeitos. Um processo não recebe
`aberto: bool` global.

## Usuário, órgão e unidade

O [índice REST](../interfaces/rest-endpoints.md) traz:
- `GET /orgao/listar`;
- `GET /usuario/unidades`;
- `GET /usuario/listar` e `GET /usuario/pesquisar`;
- `GET /contexto/listar/{orgao}`.

São interfaces documentadas, não permissões confirmadas. Conferir
os schemas completos do [OpenAPI fixado](../interfaces/rest-wssei.md)
antes de definir campos.

### Regras de identidade

- Pessoa, usuário autenticado, interessado e assinante são papéis
  diferentes; coincidência de nome não demonstra igualdade de IDs.
- Unidade pertence ao escopo de instalação/órgão; sua sigla isolada
  pode ser ambígua. Não usar sigla como identificador universal.
- Relação usuário-unidade/perfil é uma relação autorizativa; não é
  consequência da unidade ativa no momento.
- Sessão e unidade ativa são contexto de transporte/autorização,
  não propriedades fixas da pessoa nem do processo.
- O dado de origem da configuração não afirma acesso REST: cada
  operação e identidade precisa de confirmação específica.

**Teste futuro:** duas unidades com mesma sigla em órgãos distintos;
usuário com vários vínculos; pessoa interessada sem conta; troca da
unidade ativa sem alteração da instalação. Esses cenários são
discriminantes sintéticos, não alegações de que foram observados
nas cassettes atuais.

## Documento e assinatura

O [card Goiás, documento-escritas](../../cassettes/go/documento-escritas-go--16f83179ecaa.md)
indexa 69 interações HTTP, incluindo
`documento_assinar` (3) e `documento_verificar_assinatura` (3).
Cards de outras instalações também indexam `documento_assinar`.
A presença da ação não certifica uma assinatura realizada.

O [índice REST](../interfaces/rest-endpoints.md) documenta
`POST /documento/assinar`,
`GET /documento/listar/assinaturas/{documento}`,
`GET /assinante/listar` e operações próprias de blocos.

### Distinções obrigatórias

| Conceito | Não confundir com |
| --- | --- |
| Documento | Processo que o contém |
| Assinatura | Documento, bloco ou booleano `assinado` |
| Assinante | Usuário da sessão sem comprovação de identidade |
| Cargo/função exibido | Perfil ou cargo global e permanente |
| Bloco de assinatura | Envio/tramitação do processo |
| Registro visual | Validação criptográfica independente |

**Testes futuros:** documento com duas assinaturas de pessoas
distintas; assinaturas de unidades diferentes; assinatura listada
sem metadado criptográfico; bloco contendo vários documentos.
Preservar como desconhecidos os campos não presentes na fonte.

## Proposta de recorte Rust

A implementação inicial deve preservar os tipos atuais da caixa.
Somente quando a operação de leitura retornar dados reais e
verificáveis, acrescentar novos IDs tipados (`IdUnidade`,
`IdUsuario`, `IdDocumento`) vinculados ao escopo da instalação.
A estrutura de assinatura nasce do contrato de leitura das assinaturas;
a estrutura de atuação nasce da leitura de situação por unidade.
As relações do catálogo não exigem ORM, banco ou roteador genérico.

Critérios para uma PR de código:
1. Uma operação selecionada e um contrato de domínio explícito.
2. Fixture/replay que discrimine identificadores e estados.
3. Golden JSON apenas se a saída pública for intencionalmente alterada.
4. Testes negativos para identidade ambígua e ausência de permissão.
5. Nenhum segredo, URL assinada ou dado autenticado em artefatos Git.

## Limites deste avanço

Não foram baixados nem inspecionados os corpos dos ZIPs nesta etapa.
As inferências foram feitas **somente** dos cards OKF e do inventário
de endpoints presente no repositório. Não houve testes live,
compilação, escrita no SEI nem confirmação de efeitos administrativos.

Próxima ação com maior valor: abrir os passos relevantes do ZIP de
`processo-escritas-go`, reconstruir estado inicial e final por unidade,
e comparar com UnB/UFRJ antes de fixar identificadores da tramitação.
Em paralelo, inspecionar a estrutura das respostas de listagem de
assinaturas e unidades nas fontes autorizadas.
