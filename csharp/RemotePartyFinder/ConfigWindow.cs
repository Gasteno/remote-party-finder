using System;
using System.Collections.Generic;
using System.Linq;
using System.Numerics;
using System.Threading.Tasks;
using Dalamud.Interface.Components;
using Dalamud.Interface.Utility.Raii;
using Dalamud.Interface.Windowing;
using DalamudImGui = Dalamud.Bindings.ImGui;

namespace RemotePartyFinder;

public class ConfigWindow : Window, IDisposable
{
    private readonly Configuration _configuration;
    private string _uploadUrlTempString = string.Empty;
    private string _uploadUrlError = string.Empty;

    public ConfigWindow(Plugin plugin) : base("Remote Party Finder")
    {
        _configuration = plugin.Configuration;
        Flags = DalamudImGui.ImGuiWindowFlags.NoCollapse | DalamudImGui.ImGuiWindowFlags.AlwaysAutoResize;

        Size = new Vector2(500, 0);
    }

    public void Dispose()
    {
    }

    public override void OnClose()
    {
        _configuration.Save();
    }

    public override void Draw()
    {
        var isAdvanced = _configuration.AdvancedSettingsEnabled;
        DalamudImGui.ImGui.TextWrapped(
            "This section is for advanced users to configure which services to send party finder data to. " +
            "Only enable if you know what you are doing.");
        if (DalamudImGui.ImGui.Checkbox("Enable Advanced Settings", ref isAdvanced))
        {
            _configuration.AdvancedSettingsEnabled = isAdvanced;
            _configuration.Save();
        }

        if (!isAdvanced) return;

        
        using (ImRaii.Table((DalamudImGui.ImU8String)"uploadUrls", 3, DalamudImGui.ImGuiTableFlags.SizingFixedFit | DalamudImGui.ImGuiTableFlags.Borders))
        {
            DalamudImGui.ImGui.TableSetupColumn("#", DalamudImGui.ImGuiTableColumnFlags.WidthFixed);
            DalamudImGui.ImGui.TableSetupColumn("URL", DalamudImGui.ImGuiTableColumnFlags.WidthStretch);
            DalamudImGui.ImGui.TableSetupColumn("Enabled", DalamudImGui.ImGuiTableColumnFlags.WidthFixed);
            DalamudImGui.ImGui.TableHeadersRow();
            
            using var id = ImRaii.PushId((DalamudImGui.ImU8String)"urls");
            foreach (var (uploadUrl, index) in _configuration.UploadUrls.Select((url, index) => (url, index + 1)))
            {
                id.Push(index);

                DalamudImGui.ImGui.TableNextRow();
                DalamudImGui.ImGui.TableSetColumnIndex(0);
                DalamudImGui.ImGui.TextUnformatted(index.ToString());
                
                DalamudImGui.ImGui.TableSetColumnIndex(1);
                DalamudImGui.ImGui.TextUnformatted(uploadUrl.Url);

                DalamudImGui.ImGui.TableSetColumnIndex(2);
                var isEnabled = uploadUrl.IsEnabled;
                if (DalamudImGui.ImGui.Checkbox("##uploadUrlCheckbox", ref isEnabled))
                {
                    uploadUrl.IsEnabled = isEnabled;
                }

                if (!uploadUrl.IsDefault)
                {
                    DalamudImGui.ImGui.SameLine();
                    if (ImGuiComponents.IconButton(Dalamud.Interface.FontAwesomeIcon.Trash))
                    {
                        _configuration.UploadUrls = _configuration.UploadUrls.Remove(uploadUrl);
                    }
                }
                
                id.Pop();
            }
            
            DalamudImGui.ImGui.TableNextRow();
            DalamudImGui.ImGui.TableSetColumnIndex(1);
            DalamudImGui.ImGui.SetNextItemWidth(-1);
            DalamudImGui.ImGui.InputText("##uploadUrlInput", ref _uploadUrlTempString, 300);
            DalamudImGui.ImGui.TableNextColumn();

            if (!string.IsNullOrEmpty(_uploadUrlTempString) &&
                ImGuiComponents.IconButton(Dalamud.Interface.FontAwesomeIcon.Plus))
            {
                _uploadUrlTempString = _uploadUrlTempString.TrimEnd();

                if (_configuration.UploadUrls.Any(r =>
                        string.Equals(r.Url, _uploadUrlTempString, StringComparison.InvariantCultureIgnoreCase)))
                {
                    _uploadUrlError = "Endpoint already exists.";
                    Task.Delay(5000).ContinueWith(t => _uploadUrlError = string.Empty);
                }
                else if (!ValidUrl(_uploadUrlTempString))
                {
                    this._uploadUrlError = "Invalid URL format.";
                    Task.Delay(5000).ContinueWith(t => _uploadUrlError = string.Empty);
                }
                else
                {
                    _configuration.UploadUrls = _configuration.UploadUrls.Add(new(_uploadUrlTempString));
                    _uploadUrlTempString = string.Empty;
                }
            }
        }

        DalamudImGui.ImGui.Dummy(new (0, 5));

        if (DalamudImGui.ImGui.Button("Reset To Default##uploadUrlDefault"))
        {
            ResetToDefault();
        }

        if (string.IsNullOrEmpty(_uploadUrlError)) return;
        
        DalamudImGui.ImGui.SameLine();
        DalamudImGui.ImGui.TextColored(new Vector4(1, 0, 0, 1), _uploadUrlError);
    }

    private void ResetToDefault()
    {
        _configuration.UploadUrls = Configuration.DefaultUploadUrls();
        _configuration.Save();
    }

    private static bool ValidUrl(string url)
        => Uri.TryCreate(url, UriKind.Absolute, out var uriResult)
           && (uriResult.Scheme == Uri.UriSchemeHttps || uriResult.Scheme == Uri.UriSchemeHttp);
}